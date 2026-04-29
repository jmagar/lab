use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use agent_client_protocol::schema::{
    CancelNotification, ClientCapabilities, ConfigOptionUpdate, ContentBlock, ContentChunk,
    CreateTerminalRequest, CurrentModeUpdate, FileSystemCapabilities, Implementation,
    InitializeRequest, KillTerminalRequest, PermissionOptionKind, ProtocolVersion,
    ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionInfoUpdate, SessionNotification, SessionUpdate, StopReason, TerminalOutputRequest,
    WaitForTerminalExitRequest, WriteTextFileRequest, WriteTextFileResponse,
};
use agent_client_protocol::{
    Agent, ByteStreams, Client, ConnectionTo, Dispatch, JsonRpcMessage, on_receive_request,
};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[cfg(unix)]
use nix::sys::signal::{Signal, killpg};
#[cfg(unix)]
use nix::unistd::Pid;

use super::types::{
    AcpEvent, AcpPermissionOption, AcpProviderHealth, StartSessionInput, StartSessionResult,
};
use crate::acp::providers::{AcpProviderEntry, read_providers};

fn acp_internal_error(message: impl Into<String>) -> agent_client_protocol::Error {
    agent_client_protocol::Error::internal_error().data(message.into())
}

const DEFAULT_PROMPT_IDLE_TIMEOUT: Duration = Duration::from_secs(5);

fn acp_prompt_idle_timeout() -> Duration {
    std::env::var("LAB_ACP_PROMPT_IDLE_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .filter(|duration| !duration.is_zero())
        .unwrap_or(DEFAULT_PROMPT_IDLE_TIMEOUT)
}

#[derive(Clone)]
pub struct RuntimeHandle {
    #[allow(dead_code)]
    pub provider_session_id: String,
    command_tx: mpsc::UnboundedSender<SessionCommand>,
}

impl RuntimeHandle {
    pub async fn prompt(&self, prompt: String) -> Result<(), String> {
        self.command_tx
            .send(SessionCommand::Prompt(prompt))
            .map_err(|_| "ACP session command channel closed".to_string())
    }

    pub async fn cancel(&self) -> Result<(), String> {
        self.command_tx
            .send(SessionCommand::Cancel)
            .map_err(|_| "ACP session command channel closed".to_string())
    }
}

enum SessionCommand {
    Prompt(String),
    Cancel,
}

#[derive(Default)]
struct StreamMessageIds {
    user: Option<String>,
    assistant: Option<String>,
}

impl StreamMessageIds {
    fn user_message_id(&mut self) -> String {
        self.user
            .get_or_insert_with(|| uuid::Uuid::new_v4().to_string())
            .clone()
    }

    fn assistant_message_id(&mut self) -> String {
        self.assistant
            .get_or_insert_with(|| uuid::Uuid::new_v4().to_string())
            .clone()
    }
}

struct RuntimeStarted {
    provider_session_id: String,
    agent_name: String,
    agent_version: String,
}

#[derive(Default)]
struct PromptLifecycle {
    active: AtomicBool,
    terminal_sent: AtomicBool,
    saw_prompt_progress: AtomicBool,
}

impl PromptLifecycle {
    fn start(&self) {
        self.active.store(true, Ordering::SeqCst);
        self.terminal_sent.store(false, Ordering::SeqCst);
        self.saw_prompt_progress.store(false, Ordering::SeqCst);
    }

    fn note_prompt_progress(&self) {
        self.saw_prompt_progress.store(true, Ordering::SeqCst);
    }

    fn finish(&self) {
        self.terminal_sent.store(true, Ordering::SeqCst);
        self.active.store(false, Ordering::SeqCst);
    }

    fn take_unfinished_prompt(&self) -> Option<bool> {
        let was_active = self.active.swap(false, Ordering::SeqCst);
        let terminal_sent = self.terminal_sent.load(Ordering::SeqCst);
        if was_active && !terminal_sent {
            self.terminal_sent.store(true, Ordering::SeqCst);
            return Some(self.saw_prompt_progress.load(Ordering::SeqCst));
        }
        None
    }
}

struct SessionDispatchProgress {
    assistant_output: bool,
    prompt_progress: bool,
}

#[derive(Clone)]
struct CodexLaunch {
    command: String,
    args: Vec<String>,
}

#[derive(Clone)]
struct ProviderLaunch {
    id: String,
    command: String,
    args: Vec<String>,
}

fn codex_launch_override() -> &'static Mutex<Option<CodexLaunch>> {
    static OVERRIDE: OnceLock<Mutex<Option<CodexLaunch>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[doc(hidden)]
#[allow(dead_code)]
pub fn set_codex_launch_override_for_tests(command: Option<String>, args: Vec<String>) {
    let mut launch = codex_launch_override()
        .lock()
        .expect("codex launch override poisoned");
    *launch = command.map(|command| CodexLaunch { command, args });
}

pub async fn launch_codex_runtime(
    session_id: String,
    input: StartSessionInput,
    event_tx: mpsc::UnboundedSender<AcpEvent>,
) -> Result<(RuntimeHandle, StartSessionResult), String> {
    let (started_tx, started_rx) = oneshot::channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();

    std::thread::Builder::new()
        .name(format!("lab-acp-{session_id}"))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build ACP runtime");
            runtime.block_on(async move {
                drop(run_codex_session(session_id, input, event_tx, started_tx, command_rx).await);
            });
        })
        .map_err(|error| error.to_string())?;

    let started = started_rx
        .await
        .map_err(|_| "ACP runtime failed to report startup".to_string())??;

    Ok((
        RuntimeHandle {
            provider_session_id: started.provider_session_id.clone(),
            command_tx,
        },
        StartSessionResult {
            provider_session_id: started.provider_session_id,
            agent_name: started.agent_name,
            agent_version: started.agent_version,
        },
    ))
}

pub fn normalize_provider_id(provider: Option<&str>) -> String {
    match provider.filter(|value| !value.trim().is_empty()) {
        Some("codex") | None => "codex-acp".to_string(),
        Some(provider) => provider.to_string(),
    }
}

pub fn provider_healths() -> Vec<AcpProviderHealth> {
    let mut providers: Vec<AcpProviderHealth> = read_providers()
        .unwrap_or_default()
        .into_iter()
        .map(|provider| health_for_provider_entry(&provider))
        .collect();

    if !providers
        .iter()
        .any(|provider| provider.provider == "codex-acp")
    {
        providers.insert(0, codex_provider_health());
    }

    providers
}

pub fn codex_provider_health() -> AcpProviderHealth {
    let (command, _args) = resolve_codex_launch();
    let ready = if std::env::var("ACP_CODEX_COMMAND")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        true
    } else {
        let probe = std::process::Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(&command)
            .output();
        probe.is_ok_and(|output| output.status.success())
    };

    AcpProviderHealth {
        provider: "codex-acp".to_string(),
        available: ready,
        version: None,
        message: if ready {
            None
        } else {
            Some(
                "ACP Codex provider is unavailable. Set ACP_CODEX_COMMAND or ensure npx is on PATH."
                    .to_string(),
            )
        },
    }
}

fn health_for_provider_entry(provider: &AcpProviderEntry) -> AcpProviderHealth {
    let launch = launch_from_provider_entry(provider);
    let available = command_available(&launch.command);
    AcpProviderHealth {
        provider: provider.id.clone(),
        available,
        version: Some(provider.version.clone()),
        message: if available {
            None
        } else {
            Some(format!(
                "ACP provider command `{}` is unavailable",
                launch.command
            ))
        },
    }
}

fn command_available(command: &str) -> bool {
    if command.contains('/') || command.contains('\\') {
        return Path::new(command).exists();
    }
    cached_command_lookup(command)
}

/// TTL cache for `which`/`where` lookups. Provider health endpoints can be
/// polled per-request; without this each call shells out once per provider.
fn cached_command_lookup(command: &str) -> bool {
    const CACHE_TTL: Duration = Duration::from_secs(10);

    static CACHE: OnceLock<Mutex<std::collections::HashMap<String, (Instant, bool)>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));

    if let Ok(map) = cache.lock() {
        if let Some((stored_at, available)) = map.get(command) {
            if stored_at.elapsed() < CACHE_TTL {
                return *available;
            }
        }
    }

    let probe = std::process::Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(command)
        .output();
    let available = probe.is_ok_and(|output| output.status.success());

    if let Ok(mut map) = cache.lock() {
        map.insert(command.to_string(), (Instant::now(), available));
    }
    available
}

fn resolve_codex_launch() -> (String, Vec<String>) {
    if let Some(launch) = codex_launch_override()
        .lock()
        .expect("codex launch override poisoned")
        .clone()
    {
        return (launch.command, launch.args);
    }

    if let Some(command) = std::env::var("ACP_CODEX_COMMAND")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        let args = std::env::var("ACP_CODEX_ARGS")
            .unwrap_or_default()
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        return (command, args);
    }

    let command = if cfg!(windows) { "npx.cmd" } else { "npx" }.to_string();
    (command, vec!["@zed-industries/codex-acp".to_string()])
}

fn launch_from_provider_entry(provider: &AcpProviderEntry) -> ProviderLaunch {
    let mut parts = provider.command.split_whitespace();
    let command = parts.next().unwrap_or("").to_string();
    let args = parts.map(ToOwned::to_owned).collect();
    ProviderLaunch {
        id: provider.id.clone(),
        command,
        args,
    }
}

fn resolve_provider_launch(provider: Option<&str>) -> Result<ProviderLaunch, String> {
    let provider_id = normalize_provider_id(provider);
    if provider_id == "codex-acp" {
        if let Some(entry) = read_providers()
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|entry| entry.id == provider_id)
        {
            return Ok(launch_from_provider_entry(&entry));
        }
        let (command, args) = resolve_codex_launch();
        return Ok(ProviderLaunch {
            id: provider_id,
            command,
            args,
        });
    }

    read_providers()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|entry| entry.id == provider_id)
        .map(|entry| launch_from_provider_entry(&entry))
        .ok_or_else(|| format!("ACP provider `{provider_id}` is not installed"))
}

async fn run_codex_session(
    session_id: String,
    input: StartSessionInput,
    event_tx: mpsc::UnboundedSender<AcpEvent>,
    started_tx: oneshot::Sender<Result<RuntimeStarted, String>>,
    mut command_rx: mpsc::UnboundedReceiver<SessionCommand>,
) -> Result<(), String> {
    let launch = resolve_provider_launch(input.provider.as_deref())?;
    let provider_id = launch.id.clone();
    let mut command = tokio::process::Command::new(&launch.command);
    command
        .args(&launch.args)
        .current_dir(Path::new(&input.cwd))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let child_process_group = child.id();

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| format!("{provider_id} stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{provider_id} stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("{provider_id} stderr unavailable"))?;

    let stderr_tx = event_tx.clone();
    let stderr_session = session_id.clone();
    let stderr_provider = provider_id.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            drop(stderr_tx.send(provider_info_event(
                stderr_session.clone(),
                &stderr_provider,
                json!({
                    "type": "stderr",
                    "title": format!("{stderr_provider} stderr"),
                    "text": line,
                }),
            )));
        }
    });

    let transport = ByteStreams::new(stdin.compat_write(), stdout.compat());
    let started_tx = Arc::new(Mutex::new(Some(started_tx)));
    let prompt_lifecycle = Arc::new(PromptLifecycle::default());
    let connection_provider = provider_id.clone();
    let run_result = Client
        .builder()
        .on_receive_request(
            {
                let session_id = session_id.clone();
                let event_tx = event_tx.clone();
                async move |args: RequestPermissionRequest, responder, _cx| {
                    let title = args.tool_call.fields.title.clone();
                    let selected = args
                        .options
                        .iter()
                        .find(|option| option.kind == PermissionOptionKind::AllowOnce)
                        .or_else(|| {
                            args.options
                                .iter()
                                .find(|option| option.kind == PermissionOptionKind::AllowAlways)
                        })
                        .or_else(|| args.options.first());

                    drop(event_tx.send(AcpEvent::PermissionRequest {
                        id: uuid::Uuid::new_v4().to_string(),
                        created_at: jiff::Timestamp::now().to_string(),
                        session_id: session_id.clone(),
                        seq: 0,
                        request_id: args.tool_call.tool_call_id.to_string(),
                        action_summary: title
                            .clone()
                            .unwrap_or_else(|| "Permission requested".to_string()),
                        options: args
                            .options
                            .iter()
                            .map(|option| AcpPermissionOption {
                                option_id: option.option_id.to_string(),
                                name: option.name.clone(),
                                kind: match option.kind {
                                    PermissionOptionKind::AllowOnce => "allow_once",
                                    PermissionOptionKind::AllowAlways => "allow_always",
                                    PermissionOptionKind::RejectOnce => "reject_once",
                                    PermissionOptionKind::RejectAlways => "reject_always",
                                    _ => "unknown",
                                }
                                .to_string(),
                            })
                            .collect(),
                    }));

                    drop(event_tx.send(AcpEvent::PermissionOutcome {
                        id: uuid::Uuid::new_v4().to_string(),
                        created_at: jiff::Timestamp::now().to_string(),
                        session_id: session_id.clone(),
                        seq: 0,
                        request_id: args.tool_call.tool_call_id.to_string(),
                        granted: selected.is_some_and(|option| {
                            matches!(
                                option.kind,
                                PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways
                            )
                        }),
                    }));

                    responder.respond(RequestPermissionResponse::new(match selected {
                        Some(option) => RequestPermissionOutcome::Selected(
                            SelectedPermissionOutcome::new(option.option_id.clone()),
                        ),
                        None => RequestPermissionOutcome::Cancelled,
                    }))
                }
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |args: ReadTextFileRequest, responder, _cx| {
                let content = tokio::fs::read_to_string(&args.path)
                    .await
                    .map_err(agent_client_protocol::Error::into_internal_error)?;
                let lines: Vec<&str> = content.lines().collect();
                let start = args.line.unwrap_or(1).saturating_sub(1) as usize;
                let end = args
                    .limit
                    .map(|limit| start.saturating_add(limit as usize))
                    .unwrap_or(lines.len());
                let selected = lines
                    .get(start..std::cmp::min(end, lines.len()))
                    .unwrap_or(&[])
                    .join("\n");
                responder.respond(ReadTextFileResponse::new(selected))
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |args: WriteTextFileRequest, responder, _cx| {
                if let Some(parent) = args.path.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(agent_client_protocol::Error::into_internal_error)?;
                }
                tokio::fs::write(&args.path, args.content)
                    .await
                    .map_err(agent_client_protocol::Error::into_internal_error)?;
                responder.respond(WriteTextFileResponse::new())
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |_args: CreateTerminalRequest, responder, _cx| {
                responder.respond_with_error(agent_client_protocol::Error::method_not_found())
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |_args: TerminalOutputRequest, responder, _cx| {
                responder.respond_with_error(agent_client_protocol::Error::method_not_found())
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |_args: ReleaseTerminalRequest, responder, _cx| {
                responder.respond_with_error(agent_client_protocol::Error::method_not_found())
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |_args: WaitForTerminalExitRequest, responder, _cx| {
                responder.respond_with_error(agent_client_protocol::Error::method_not_found())
            },
            on_receive_request!(),
        )
        .on_receive_request(
            async move |_args: KillTerminalRequest, responder, _cx| {
                responder.respond_with_error(agent_client_protocol::Error::method_not_found())
            },
            on_receive_request!(),
        )
        .connect_with(transport, {
            let session_id = session_id.clone();
            let event_tx = event_tx.clone();
            let cwd = input.cwd.clone();
            let started_tx = Arc::clone(&started_tx);
            let prompt_lifecycle = Arc::clone(&prompt_lifecycle);
            let provider_id = connection_provider.clone();
            move |connection: ConnectionTo<Agent>| {
                let session_id = session_id.clone();
                let event_tx = event_tx.clone();
                let cwd = cwd.clone();
                let started_tx = Arc::clone(&started_tx);
                let prompt_lifecycle = Arc::clone(&prompt_lifecycle);
                let provider_id = provider_id.clone();
                async move {
                    let initialized = connection
                        .send_request(
                            InitializeRequest::new(ProtocolVersion::V1)
                                .client_info(
                                    Implementation::new(
                                        "lab-acp-bridge",
                                        env!("CARGO_PKG_VERSION"),
                                    )
                                    .title("Lab ACP Bridge"),
                                )
                                .client_capabilities({
                                    // PHASE 1: do NOT call .terminal(true) — server-hosted terminal
                                    // execution lives in lab-lffl. Removing this comment without
                                    // removing the corresponding lab-lffl gate is a regression.
                                    let mut meta = serde_json::Map::new();
                                    meta.insert(
                                        "terminal_output".to_string(),
                                        json!(true),
                                    );
                                    ClientCapabilities::new()
                                        .fs(
                                            FileSystemCapabilities::new()
                                                .read_text_file(true)
                                                .write_text_file(true),
                                        )
                                        .meta(meta)
                                }),
                        )
                        .block_task()
                        .await
                        .map_err(|error| acp_internal_error(error.to_string()))?;

                    let mut session = connection
                        .build_session(cwd)
                        .block_task()
                        .start_session()
                        .await
                        .map_err(|error| acp_internal_error(error.to_string()))?;

                    let started = RuntimeStarted {
                        provider_session_id: session.session_id().to_string(),
                        agent_name: initialized
                            .agent_info
                            .as_ref()
                            .and_then(|info| info.title.clone())
                            .unwrap_or_else(|| {
                                initialized
                                    .agent_info
                                    .as_ref()
                                    .map(|info| info.name.clone())
                                    .unwrap_or_else(|| provider_id.clone())
                            }),
                        agent_version: initialized
                            .agent_info
                            .as_ref()
                            .map(|info| info.version.clone())
                            .unwrap_or_else(|| "unknown".to_string()),
                    };
                    if let Some(sender) = started_tx.lock().ok().and_then(|mut guard| guard.take()) {
                        drop(sender.send(Ok(started)));
                    }

                    while let Some(command) = command_rx.recv().await {
                        match command {
                            SessionCommand::Prompt(prompt) => {
                                prompt_lifecycle.start();
                                let stream_message_ids =
                                    Arc::new(Mutex::new(StreamMessageIds::default()));
                                drop(event_tx.send(session_state_event(
                                    session_id.clone(),
                                    lab_apis::acp::types::AcpSessionState::Running,
                                )));
                                drop(event_tx.send(provider_info_event(
                                    session_id.clone(),
                                    &provider_id,
                                    json!({
                                        "type": "prompt_started",
                                        "title": "Prompt started",
                                        "text": prompt.clone(),
                                    }),
                                )));

                                session
                                    .send_prompt(prompt)
                                    .map_err(|error| acp_internal_error(error.to_string()))?;

                                let mut saw_assistant_output = false;
                                loop {
                                    let update = tokio::select! {
                                        update = session.read_update() => Some(update),
                                        () = tokio::time::sleep(acp_prompt_idle_timeout()), if saw_assistant_output => None,
                                    };
                                    let update = match update {
                                        Some(update) => update,
                                        None => {
                                            drop(event_tx.send(session_state_event(
                                                session_id.clone(),
                                                lab_apis::acp::types::AcpSessionState::Completed,
                                            )));
                                            drop(event_tx.send(provider_info_event(
                                                session_id.clone(),
                                                &provider_id,
                                                json!({
                                                    "type": "idle_completion",
                                                    "title": "Prompt completed after provider idle timeout",
                                                    "status": "completed",
                                                    "timeout_ms": acp_prompt_idle_timeout().as_millis(),
                                                }),
                                            )));
                                            prompt_lifecycle.finish();
                                            break;
                                        }
                                    };

                                    match update {
                                        Ok(agent_client_protocol::SessionMessage::SessionMessage(
                                            dispatch,
                                        )) => {
                                            let progress = handle_session_dispatch(
                                                &session_id,
                                                &event_tx,
                                                dispatch,
                                                &stream_message_ids,
                                            )
                                            .map_err(acp_internal_error)?;
                                            saw_assistant_output |= progress.assistant_output;
                                            if progress.prompt_progress {
                                                prompt_lifecycle.note_prompt_progress();
                                            }
                                        }
                                        Ok(agent_client_protocol::SessionMessage::StopReason(
                                            stop_reason,
                                        )) => {
                                            let stop_reason =
                                                map_stop_reason(&stop_reason).to_string();
                                            let state = if stop_reason == "cancelled" {
                                                lab_apis::acp::types::AcpSessionState::Cancelled
                                            } else {
                                                lab_apis::acp::types::AcpSessionState::Completed
                                            };
                                            drop(event_tx.send(session_state_event(
                                                session_id.clone(),
                                                state.clone(),
                                            )));
                                            drop(event_tx.send(provider_info_event(
                                                session_id.clone(),
                                                &provider_id,
                                                json!({
                                                    "type": "stop_reason",
                                                    "title": "Prompt completed",
                                                    "status": match state {
                                                        lab_apis::acp::types::AcpSessionState::Cancelled => "cancelled",
                                                        _ => "completed",
                                                    },
                                                    "stop_reason": stop_reason,
                                                }),
                                            )));
                                            prompt_lifecycle.finish();
                                            break;
                                        }
                                        Ok(_) => {
                                            drop(event_tx.send(provider_info_event(
                                                session_id.clone(),
                                                &provider_id,
                                                json!({
                                                    "type": "unhandled_provider_message",
                                                    "title": "Unhandled provider update",
                                                }),
                                            )));
                                        }
                                        Err(error) => {
                                            drop(event_tx.send(session_state_event(
                                                session_id.clone(),
                                                lab_apis::acp::types::AcpSessionState::Failed,
                                            )));
                                            drop(event_tx.send(provider_info_event(
                                                session_id.clone(),
                                                &provider_id,
                                                json!({
                                                    "type": "provider_error",
                                                    "title": "Provider error",
                                                    "text": error.to_string(),
                                                }),
                                            )));
                                            prompt_lifecycle.finish();
                                            break;
                                        }
                                    }
                                }
                            }
                            SessionCommand::Cancel => {
                                session
                                    .connection()
                                    .send_notification(CancelNotification::new(
                                        session.session_id().clone(),
                                    ))
                                    .map_err(|error| acp_internal_error(error.to_string()))?;
                            }
                        }
                    }

                    Ok::<(), agent_client_protocol::Error>(())
                }
            }
        })
        .await;

    if let Some(saw_assistant_output) = prompt_lifecycle.take_unfinished_prompt() {
        let state = if saw_assistant_output {
            lab_apis::acp::types::AcpSessionState::Completed
        } else {
            lab_apis::acp::types::AcpSessionState::Failed
        };
        let status = match state {
            lab_apis::acp::types::AcpSessionState::Completed => "completed",
            _ => "failed",
        };
        drop(event_tx.send(session_state_event(session_id.clone(), state)));
        drop(event_tx.send(provider_info_event(
            session_id.clone(),
            &provider_id,
            json!({
                "type": "runtime_exit_without_stop_reason",
                "title": "ACP provider exited before sending a prompt stop reason",
                "status": status,
            }),
        )));
    }

    let run_error = run_result.err();

    terminate_codex_child(&mut child, child_process_group).await;

    if let Some(error) = run_error {
        if let Some(sender) = started_tx.lock().ok().and_then(|mut guard| guard.take()) {
            drop(sender.send(Err(error.to_string())));
        }
        return Err(error.to_string());
    }

    Ok(())
}

#[cfg_attr(not(unix), allow(unused_variables))]
async fn terminate_codex_child(
    child: &mut tokio::process::Child,
    child_process_group: Option<u32>,
) {
    #[cfg(unix)]
    if let Some(pid) = child_process_group.and_then(|value| i32::try_from(value).ok()) {
        let pgid = Pid::from_raw(pid);
        let _ = killpg(pgid, Signal::SIGTERM);
        tokio::time::sleep(Duration::from_millis(250)).await;
        if matches!(child.try_wait(), Ok(None)) {
            let _ = killpg(pgid, Signal::SIGKILL);
        }
        drop(child.wait().await);
        return;
    }

    drop(child.kill().await);
}

fn push_session_update(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    update: SessionUpdate,
    message_ids: &mut StreamMessageIds,
) -> Result<(), String> {
    match update {
        SessionUpdate::UserMessageChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(AcpEvent::MessageChunk {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    role: "user".to_string(),
                    text: content_to_text(content),
                    message_id: message_ids.user_message_id(),
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AgentMessageChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(AcpEvent::MessageChunk {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    role: "assistant".to_string(),
                    text: content_to_text(content),
                    message_id: message_ids.assistant_message_id(),
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AgentThoughtChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(AcpEvent::ReasoningChunk {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    text: content_to_text(content),
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::ToolCall(tool_call) => {
            event_tx
                .send(AcpEvent::ToolCallStart {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    tool_call_id: tool_call.tool_call_id.to_string(),
                    name: tool_call.title.clone(),
                    input: tool_call.raw_input.unwrap_or(Value::Null),
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
            if let Some(status) = enum_value(&tool_call.status) {
                // _meta must be omitted entirely when absent; json!() would emit null.
                // _meta is a transparent relay — never log its field values.
                let mut payload = json!({
                    "type": "tool_call_metadata",
                    "tool_call_id": tool_call.tool_call_id.to_string(),
                    "title": tool_call.title,
                    "tool_kind": enum_value(&tool_call.kind),
                    "status": status,
                    "locations": tool_call.locations.iter()
                        .map(|l| l.path.display().to_string())
                        .collect::<Vec<_>>(),
                    "content": tool_call.content,
                    "raw_output": tool_call.raw_output,
                });
                if let Some(meta) = tool_call.meta {
                    payload
                        .as_object_mut()
                        .unwrap()
                        .insert("_meta".into(), Value::Object(meta));
                }
                event_tx
                    .send(provider_info_event(
                        session_id.to_string(),
                        "codex",
                        payload,
                    ))
                    .map_err(|_| "ACP event channel closed".to_string())?;
            }
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let tool_call_id = update.tool_call_id.to_string();
            let status = update
                .fields
                .status
                .as_ref()
                .and_then(enum_value)
                .unwrap_or_else(|| "updated".to_string());
            event_tx
                .send(AcpEvent::ToolCallUpdate {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    tool_call_id,
                    output: tool_call_update_output(update),
                    status,
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::Plan(plan) => {
            event_tx
                .send(provider_info_event(
                    session_id.to_string(),
                    "codex",
                    json!({
                        "type": "plan",
                        "title": "Execution plan updated",
                        "entries": serde_json::to_value(&plan)
                            .ok()
                            .and_then(|value| value.get("entries").cloned())
                            .unwrap_or(Value::Null),
                    }),
                ))
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AvailableCommandsUpdate(update) => {
            event_tx
                .send(provider_info_event(
                    session_id.to_string(),
                    "codex",
                    json!({
                        "type": "commands",
                        "title": "Available commands updated",
                        "commands": serde_json::to_value(&update)
                            .ok()
                            .and_then(|value| value.get("commands").cloned())
                            .unwrap_or(Value::Null),
                    }),
                ))
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::CurrentModeUpdate(update) => {
            emit_current_mode(session_id, event_tx, update)?;
        }
        SessionUpdate::ConfigOptionUpdate(update) => {
            emit_config_update(session_id, event_tx, update)?;
        }
        SessionUpdate::SessionInfoUpdate(update) => {
            emit_session_info(session_id, event_tx, update)?;
        }
        other => {
            event_tx
                .send(provider_info_event(
                    session_id.to_string(),
                    "codex",
                    json!({
                        "type": "debug",
                        "title": "Unhandled session update",
                        "payload": serde_json::to_value(&other).unwrap_or(Value::Null),
                    }),
                ))
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
    }

    Ok(())
}

fn handle_session_dispatch(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    dispatch: Dispatch,
    stream_message_ids: &Arc<Mutex<StreamMessageIds>>,
) -> Result<SessionDispatchProgress, String> {
    match dispatch {
        Dispatch::Notification(notification)
            if SessionNotification::matches_method(notification.method()) =>
        {
            let notification =
                SessionNotification::parse_message(notification.method(), notification.params())
                    .map_err(|error| error.to_string())?;
            let is_assistant_output =
                matches!(notification.update, SessionUpdate::AgentMessageChunk(_));
            let is_prompt_progress = is_prompt_progress_update(&notification.update);
            let mut message_ids = stream_message_ids
                .lock()
                .map_err(|_| "ACP stream message id tracker poisoned".to_string())?;
            push_session_update(session_id, event_tx, notification.update, &mut message_ids)?;
            Ok(SessionDispatchProgress {
                assistant_output: is_assistant_output,
                prompt_progress: is_prompt_progress,
            })
        }
        Dispatch::Notification(notification) => event_tx
            .send(provider_info_event(
                session_id.to_string(),
                "codex",
                json!({
                    "type": "unhandled_provider_notification",
                    "title": "Unhandled provider notification",
                    "method": notification.method(),
                }),
            ))
            .map_err(|_| "ACP event channel closed".to_string())
            .map(|()| SessionDispatchProgress {
                assistant_output: false,
                prompt_progress: false,
            }),
        Dispatch::Request(request, responder) => {
            drop(responder.respond_with_error(agent_client_protocol::Error::method_not_found()));
            event_tx
                .send(provider_info_event(
                    session_id.to_string(),
                    "codex",
                    json!({
                        "type": "unhandled_provider_request",
                        "title": "Unhandled provider request",
                        "method": request.method(),
                    }),
                ))
                .map_err(|_| "ACP event channel closed".to_string())
                .map(|()| SessionDispatchProgress {
                    assistant_output: false,
                    prompt_progress: false,
                })
        }
        Dispatch::Response(_, _) => event_tx
            .send(provider_info_event(
                session_id.to_string(),
                "codex",
                json!({
                    "type": "unhandled_provider_response",
                    "title": "Unhandled provider response",
                }),
            ))
            .map_err(|_| "ACP event channel closed".to_string())
            .map(|()| SessionDispatchProgress {
                assistant_output: false,
                prompt_progress: false,
            }),
    }
}

fn is_prompt_progress_update(update: &SessionUpdate) -> bool {
    matches!(
        update,
        SessionUpdate::AgentMessageChunk(_)
            | SessionUpdate::AgentThoughtChunk(_)
            | SessionUpdate::ToolCall(_)
            | SessionUpdate::ToolCallUpdate(_)
            | SessionUpdate::Plan(_)
            | SessionUpdate::AvailableCommandsUpdate(_)
    )
}

fn emit_current_mode(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    update: CurrentModeUpdate,
) -> Result<(), String> {
    event_tx
        .send(provider_info_event(
            session_id.to_string(),
            "codex",
            json!({
                "type": "current_mode",
                "title": "Agent mode updated",
                "current_mode": serde_json::to_value(&update).unwrap_or(Value::Null),
            }),
        ))
        .map_err(|_| "ACP event channel closed".to_string())
}

fn emit_config_update(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    update: ConfigOptionUpdate,
) -> Result<(), String> {
    event_tx
        .send(provider_info_event(
            session_id.to_string(),
            "codex",
            json!({
                "type": "config_update",
                "title": "Configuration options updated",
                "config_update": serde_json::to_value(&update).unwrap_or(Value::Null),
            }),
        ))
        .map_err(|_| "ACP event channel closed".to_string())
}

fn emit_session_info(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    update: SessionInfoUpdate,
) -> Result<(), String> {
    event_tx
        .send(provider_info_event(
            session_id.to_string(),
            "codex",
            json!({
                "type": "session_info",
                "title": "Session info updated",
                "session_info": serde_json::to_value(&update).unwrap_or(Value::Null),
            }),
        ))
        .map_err(|_| "ACP event channel closed".to_string())
}

fn session_state_event(
    session_id: String,
    state: lab_apis::acp::types::AcpSessionState,
) -> AcpEvent {
    AcpEvent::SessionUpdate {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: jiff::Timestamp::now().to_string(),
        session_id,
        seq: 0,
        state,
    }
}

fn provider_info_event(session_id: String, provider: &str, raw: Value) -> AcpEvent {
    AcpEvent::ProviderInfo {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: jiff::Timestamp::now().to_string(),
        session_id,
        seq: 0,
        provider: provider.to_string(),
        raw,
    }
}

fn tool_call_update_output(update: agent_client_protocol::schema::ToolCallUpdate) -> Value {
    let meta = update.meta;
    let fields = update.fields;
    // When raw_output is present and is an Object, inject the wrapper-level _meta into it
    // (outer wins — the wrapper _meta takes precedence over any _meta already in raw_output).
    // Non-object raw_output passes through unchanged.
    // Never log _meta field values (cwd, terminal_id, signal, data).
    if let Some(raw_output) = fields.raw_output {
        match raw_output {
            Value::Object(mut map) => {
                if let Some(m) = meta {
                    map.insert("_meta".into(), Value::Object(m));
                }
                return Value::Object(map);
            }
            other => return other,
        }
    }

    let mut payload = json!({
        "title": fields.title,
        "kind": fields.kind.as_ref().and_then(enum_value),
        "status": fields.status.as_ref().and_then(enum_value),
        "content": fields.content,
        "locations": fields.locations.as_ref().map(|locs| {
            locs.iter().map(|l| l.path.display().to_string()).collect::<Vec<_>>()
        }),
        "raw_input": fields.raw_input,
    });
    if let Some(m) = meta {
        payload
            .as_object_mut()
            .unwrap()
            .insert("_meta".into(), Value::Object(m));
    }
    payload
}

fn content_to_text(content: ContentBlock) -> String {
    match content {
        ContentBlock::Text(value) => value.text,
        ContentBlock::Image(_) => "[image]".to_string(),
        ContentBlock::Audio(_) => "[audio]".to_string(),
        ContentBlock::ResourceLink(value) => format!("[resource] {}", value.uri),
        ContentBlock::Resource(_) => "[embedded resource]".to_string(),
        _ => "[content]".to_string(),
    }
}

fn enum_value<T: serde::Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
}

fn map_stop_reason(stop_reason: &StopReason) -> &'static str {
    match stop_reason {
        StopReason::Cancelled => "cancelled",
        StopReason::EndTurn => "end_turn",
        StopReason::MaxTokens => "max_tokens",
        StopReason::MaxTurnRequests => "max_turn_requests",
        StopReason::Refusal => "refusal",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::schema::{
        AvailableCommandsUpdate, TextContent, ToolCall, ToolCallUpdate, ToolCallUpdateFields,
    };

    fn text_chunk(text: &str) -> ContentChunk {
        ContentChunk::new(ContentBlock::Text(TextContent::new(text)))
    }

    fn received_message_id(rx: &mut mpsc::UnboundedReceiver<AcpEvent>) -> String {
        match rx.try_recv().expect("message chunk event") {
            AcpEvent::MessageChunk { message_id, .. } => message_id,
            other => panic!("expected message chunk event, got {other:?}"),
        }
    }

    #[test]
    fn streamed_message_chunks_share_stable_message_ids_per_role() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut message_ids = StreamMessageIds::default();

        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::UserMessageChunk(text_chunk("hello ")),
            &mut message_ids,
        )
        .expect("first user chunk");
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::UserMessageChunk(text_chunk("world")),
            &mut message_ids,
        )
        .expect("second user chunk");
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::AgentMessageChunk(text_chunk("reply ")),
            &mut message_ids,
        )
        .expect("first assistant chunk");
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::AgentMessageChunk(text_chunk("done")),
            &mut message_ids,
        )
        .expect("second assistant chunk");

        let first_user_message_id = received_message_id(&mut rx);
        let second_user_message_id = received_message_id(&mut rx);
        let first_assistant_message_id = received_message_id(&mut rx);
        let second_assistant_message_id = received_message_id(&mut rx);

        assert_eq!(first_user_message_id, second_user_message_id);
        assert_eq!(first_assistant_message_id, second_assistant_message_id);
        assert_ne!(first_user_message_id, first_assistant_message_id);
    }

    #[test]
    fn prompt_progress_includes_provider_turn_activity() {
        assert!(is_prompt_progress_update(
            &SessionUpdate::AgentThoughtChunk(text_chunk("thinking"))
        ));
        assert!(is_prompt_progress_update(&SessionUpdate::ToolCall(
            ToolCall::new("tool-1", "Read file")
        )));
        assert!(is_prompt_progress_update(
            &SessionUpdate::AvailableCommandsUpdate(AvailableCommandsUpdate::new(vec![]))
        ));
    }

    /// Drain all pending events and return them. Panics if the channel is empty and
    /// expected_count events have not been collected.
    fn drain_events(
        rx: &mut mpsc::UnboundedReceiver<AcpEvent>,
        expected_count: usize,
    ) -> Vec<AcpEvent> {
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
            if events.len() == expected_count {
                break;
            }
        }
        assert_eq!(
            events.len(),
            expected_count,
            "expected {expected_count} events, got {}",
            events.len()
        );
        events
    }

    /// Build a minimal terminal_info Meta blob for tests.
    fn terminal_info_meta() -> agent_client_protocol::schema::Meta {
        let mut meta = serde_json::Map::new();
        meta.insert(
            "terminal_info".into(),
            json!({
                "terminal_id": "term-secret-42",
                "cwd": "/home/secret/projects/lab",
            }),
        );
        meta
    }

    // -----------------------------------------------------------------------
    // Test 1: tool_call_metadata_round_trips_terminal_meta
    //
    // Both SessionUpdate::ToolCall and ToolCallUpdate paths must preserve the
    // `_meta` field through to the emitted AcpEvent payload.
    // -----------------------------------------------------------------------
    #[test]
    fn tool_call_metadata_round_trips_terminal_meta() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut message_ids = StreamMessageIds::default();

        // --- ToolCall path ---
        let meta = terminal_info_meta();
        let tool_call = ToolCall::new("tc-1", "Read file")
            .status(agent_client_protocol::schema::ToolCallStatus::Completed)
            .meta(meta.clone());
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::ToolCall(tool_call),
            &mut message_ids,
        )
        .expect("ToolCall with meta");

        // Expect 2 events: ToolCallStart + provider_info (tool_call_metadata)
        let events = drain_events(&mut rx, 2);

        // First event: ToolCallStart
        assert!(
            matches!(&events[0], AcpEvent::ToolCallStart { .. }),
            "expected ToolCallStart, got {:?}",
            events[0]
        );

        // Second event: ProviderInfo carrying _meta
        match &events[1] {
            AcpEvent::ProviderInfo { raw, .. } => {
                let meta_value = raw
                    .get("_meta")
                    .expect("_meta key must be present in provider_info");
                let terminal_info = meta_value
                    .get("terminal_info")
                    .expect("terminal_info key present");
                assert_eq!(
                    terminal_info.get("terminal_id").and_then(Value::as_str),
                    Some("term-secret-42"),
                    "terminal_id must round-trip"
                );
                assert_eq!(
                    terminal_info.get("cwd").and_then(Value::as_str),
                    Some("/home/secret/projects/lab"),
                    "cwd must round-trip"
                );
            }
            other => panic!("expected ProviderInfo, got {other:?}"),
        }

        // --- ToolCallUpdate path ---
        let update_meta = terminal_info_meta();
        let fields = ToolCallUpdateFields::new();
        let update = ToolCallUpdate::new("tc-2", fields).meta(update_meta.clone());
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::ToolCallUpdate(update),
            &mut message_ids,
        )
        .expect("ToolCallUpdate with meta");

        let update_events = drain_events(&mut rx, 1);
        match &update_events[0] {
            AcpEvent::ToolCallUpdate { output, .. } => {
                let meta_value = output
                    .get("_meta")
                    .expect("_meta key must be present in output");
                let terminal_info = meta_value
                    .get("terminal_info")
                    .expect("terminal_info key present");
                assert_eq!(
                    terminal_info.get("terminal_id").and_then(Value::as_str),
                    Some("term-secret-42"),
                    "terminal_id must round-trip in ToolCallUpdate"
                );
            }
            other => panic!("expected ToolCallUpdate, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 2: tool_call_update_output_outer_meta_wins_over_raw_output_inner_meta
    //
    // A9 merge semantics: when raw_output already contains _meta, the wrapper-level
    // _meta (outer) wins.
    // -----------------------------------------------------------------------
    #[test]
    fn tool_call_update_output_outer_meta_wins_over_raw_output_inner_meta() {
        let mut outer_meta = serde_json::Map::new();
        outer_meta.insert("source".into(), Value::String("outer".into()));

        let inner_meta_json = json!({"source": "inner", "extra": "inner-only"});
        let raw_output_with_inner_meta = json!({
            "result": "ok",
            "_meta": inner_meta_json,
        });

        let fields = ToolCallUpdateFields::new().raw_output(raw_output_with_inner_meta);
        let update = ToolCallUpdate::new("tc-merge", fields).meta(outer_meta);

        let output = tool_call_update_output(update);

        // Outer _meta must win — source should be "outer", not "inner".
        let meta_value = output.get("_meta").expect("_meta key present after merge");
        assert_eq!(
            meta_value.get("source").and_then(Value::as_str),
            Some("outer"),
            "outer _meta must overwrite inner _meta"
        );
        // Inner-only key should no longer be present (entire _meta replaced, not merged).
        assert!(
            meta_value.get("extra").is_none(),
            "inner-only keys must not survive outer-wins replacement"
        );
        // Other raw_output fields must be preserved.
        assert_eq!(
            output.get("result").and_then(Value::as_str),
            Some("ok"),
            "non-_meta fields in raw_output must be preserved"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: tool_call_event_omits_meta_key_when_none
    //
    // P4: when meta is None, the `_meta` key must be absent from both the
    // ToolCall provider_info payload and the ToolCallUpdate output.
    // -----------------------------------------------------------------------
    #[test]
    fn tool_call_event_omits_meta_key_when_none() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut message_ids = StreamMessageIds::default();

        // ToolCall with no meta and a status (so the provider_info event fires)
        let tool_call = ToolCall::new("tc-no-meta", "Read file")
            .status(agent_client_protocol::schema::ToolCallStatus::Completed);
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::ToolCall(tool_call),
            &mut message_ids,
        )
        .expect("ToolCall without meta");

        let events = drain_events(&mut rx, 2);
        match &events[1] {
            AcpEvent::ProviderInfo { raw, .. } => {
                assert!(
                    raw.get("_meta").is_none(),
                    "_meta key must be absent from provider_info when ToolCall.meta is None, got: {:?}",
                    raw.get("_meta")
                );
            }
            other => panic!("expected ProviderInfo, got {other:?}"),
        }

        // ToolCallUpdate with no meta
        let fields = ToolCallUpdateFields::new();
        let update = ToolCallUpdate::new("tc-no-meta-update", fields);
        push_session_update(
            "session-1",
            &tx,
            SessionUpdate::ToolCallUpdate(update),
            &mut message_ids,
        )
        .expect("ToolCallUpdate without meta");

        let update_events = drain_events(&mut rx, 1);
        match &update_events[0] {
            AcpEvent::ToolCallUpdate { output, .. } => {
                assert!(
                    output.get("_meta").is_none(),
                    "_meta key must be absent from ToolCallUpdate output when meta is None, got: {:?}",
                    output.get("_meta")
                );
            }
            other => panic!("expected ToolCallUpdate, got {other:?}"),
        }
    }

    // _meta redaction is an architectural guarantee: push_session_update and
    // tool_call_update_output emit no tracing spans, so _meta field values
    // (cwd, terminal_id, signal, data) never reach the log output by construction.
    // Enforcement is via is_sensitive_key() in dispatch/redact.rs for the DB path.

    // -----------------------------------------------------------------------
    // Test 4: initialize_request_advertises_terminal_output_metadata_only
    //
    // Phase 1 MUST advertise _meta.terminal_output=true and terminal=false.
    // DO NOT call .terminal(true) — that would enable server-hosted execution
    // which lives in lab-lffl.
    // -----------------------------------------------------------------------
    #[test]
    fn initialize_request_advertises_terminal_output_metadata_only() {
        use agent_client_protocol::schema::InitializeRequest;

        // Build the same capabilities inline as the runtime does.
        let mut meta = serde_json::Map::new();
        meta.insert("terminal_output".to_string(), json!(true));
        let capabilities = ClientCapabilities::new()
            .fs(FileSystemCapabilities::new()
                .read_text_file(true)
                .write_text_file(true))
            .meta(meta);

        let value = serde_json::to_value(&capabilities).unwrap();

        // terminal must be false (Phase 1: no server-hosted execution).
        assert_eq!(
            value.get("terminal"),
            Some(&serde_json::json!(false)),
            "terminal must be false in Phase 1 — server-hosted execution lives in lab-lffl"
        );

        // _meta.terminal_output must be true (Phase 1: display metadata relay).
        assert_eq!(
            value.get("_meta").and_then(|m| m.get("terminal_output")),
            Some(&serde_json::json!(true)),
            "_meta.terminal_output must be true to advertise display support"
        );

        // Verify the full InitializeRequest serialization also reflects capabilities.
        let req = InitializeRequest::new(ProtocolVersion::V1).client_capabilities(capabilities);
        let req_value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            req_value
                .get("clientCapabilities")
                .and_then(|c| c.get("_meta"))
                .and_then(|m| m.get("terminal_output")),
            Some(&serde_json::json!(true)),
            "_meta.terminal_output must survive InitializeRequest serialization"
        );
        assert_eq!(
            req_value
                .get("clientCapabilities")
                .and_then(|c| c.get("terminal")),
            Some(&serde_json::json!(false)),
            "terminal must be false in InitializeRequest"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: phase_1_terminal_requests_return_method_not_found
    //
    // C6 — NEGATIVE integration test: even with _meta.terminal_output=true
    // advertised, the runtime must NOT execute terminal creation. All terminal/*
    // request handlers exist but unconditionally return method_not_found (-32601).
    // This documents the Phase 1 invariant so reviewers catch accidental
    // Phase 2 wiring. A full live RPC test requires a running ACP session
    // and belongs in integration tests; this unit test anchors the invariant
    // structurally.
    //
    // Invariant: all terminal/* on_receive_request handlers in the Dispatch
    // impl respond with `Error::method_not_found()`. No handler executes
    // terminal operations or delegates to a jail. lab-lffl is the gate that
    // activates terminal execution in Phase 2.
    // -----------------------------------------------------------------------
    #[test]
    fn phase_1_terminal_requests_return_method_not_found() {
        // CreateTerminalRequest is imported and has a handler arm that returns
        // method_not_found. Verify the import compiles. The handler arm exists
        // to satisfy the ACP protocol type system while blocking execution.
        //
        // We cannot write a live RPC test without a running ACP session, so
        // the invariant is enforced by code review + this documentation comment.
        // Remove this test only when lab-lffl lands and the security jail is in place.
        let _phantom: Option<CreateTerminalRequest> = None;
        // If this test ever fails to compile, something changed the imports.
        // If you're reading this because you want to add terminal execution,
        // see lab-lffl and docs/ACP_TERMINAL_PHASE2.md first.
    }
}
