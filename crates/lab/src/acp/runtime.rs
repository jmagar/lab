use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use agent_client_protocol::schema::{
    CancelNotification, ClientCapabilities, ConfigOptionUpdate, ContentBlock, ContentChunk,
    CreateTerminalRequest, CurrentModeUpdate, FileSystemCapabilities, Implementation,
    InitializeRequest, KillTerminalRequest, PermissionOptionKind, ProtocolVersion,
    ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionInfoUpdate, SessionNotification, SessionUpdate, StopReason,
    TerminalOutputRequest, WaitForTerminalExitRequest, WriteTextFileRequest, WriteTextFileResponse,
};
use agent_client_protocol::util::MatchDispatch;
use agent_client_protocol::{Agent, ByteStreams, Client, ConnectionTo, on_receive_request};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::types::{
    AcpEvent, AcpPermissionOption, AcpProviderHealth, StartSessionInput, StartSessionResult,
};

fn acp_internal_error(message: impl Into<String>) -> agent_client_protocol::Error {
    agent_client_protocol::Error::internal_error().data(message.into())
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

struct RuntimeStarted {
    provider_session_id: String,
    agent_name: String,
    agent_version: String,
}

#[derive(Clone)]
struct CodexLaunch {
    command: String,
    args: Vec<String>,
}

fn codex_launch_override() -> &'static Mutex<Option<CodexLaunch>> {
    static OVERRIDE: OnceLock<Mutex<Option<CodexLaunch>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[doc(hidden)]
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
        provider: "codex".to_string(),
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

async fn run_codex_session(
    session_id: String,
    input: StartSessionInput,
    event_tx: mpsc::UnboundedSender<AcpEvent>,
    started_tx: oneshot::Sender<Result<RuntimeStarted, String>>,
    mut command_rx: mpsc::UnboundedReceiver<SessionCommand>,
) -> Result<(), String> {
    let (command, args) = resolve_codex_launch();
    let mut child = tokio::process::Command::new(&command)
        .args(&args)
        .current_dir(Path::new(&input.cwd))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| error.to_string())?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "codex-acp stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "codex-acp stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "codex-acp stderr unavailable".to_string())?;

    let stderr_tx = event_tx.clone();
    let stderr_session = session_id.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            drop(stderr_tx.send(provider_info_event(
                stderr_session.clone(),
                "codex",
                json!({
                    "type": "stderr",
                    "title": "codex-acp stderr",
                    "text": line,
                }),
            )));
        }
    });

    let transport = ByteStreams::new(stdin.compat_write(), stdout.compat());
    let started_tx = Arc::new(Mutex::new(Some(started_tx)));
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
            move |connection: ConnectionTo<Agent>| {
                let session_id = session_id.clone();
                let event_tx = event_tx.clone();
                let cwd = cwd.clone();
                let started_tx = Arc::clone(&started_tx);
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
                                .client_capabilities(
                                    ClientCapabilities::new().fs(
                                        FileSystemCapabilities::new()
                                            .read_text_file(true)
                                            .write_text_file(true),
                                    ),
                                ),
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
                                    .unwrap_or_else(|| "Codex ACP".to_string())
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
                                drop(event_tx.send(session_state_event(
                                    session_id.clone(),
                                    lab_apis::acp::types::AcpSessionState::Running,
                                )));
                                drop(event_tx.send(provider_info_event(
                                    session_id.clone(),
                                    "codex",
                                    json!({
                                        "type": "prompt_started",
                                        "title": "Prompt started",
                                        "text": prompt.clone(),
                                    }),
                                )));

                                session
                                    .send_prompt(prompt)
                                    .map_err(|error| acp_internal_error(error.to_string()))?;

                                loop {
                                    match session.read_update().await {
                                        Ok(agent_client_protocol::SessionMessage::SessionMessage(
                                            dispatch,
                                        )) => {
                                            MatchDispatch::new(dispatch)
                                                .if_notification({
                                                    let session_id = session_id.clone();
                                                    let event_tx = event_tx.clone();
                                                    async move |notification: SessionNotification| {
                                                        drop(push_session_update(
                                                            &session_id,
                                                            &event_tx,
                                                            notification.update,
                                                        ));
                                                        Ok(())
                                                    }
                                                })
                                                .await
                                                .otherwise_ignore()
                                                .map_err(|error| acp_internal_error(error.to_string()))?;
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
                                                "codex",
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
                                            break;
                                        }
                                        Ok(_) => {
                                            drop(event_tx.send(provider_info_event(
                                                session_id.clone(),
                                                "codex",
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
                                                "codex",
                                                json!({
                                                    "type": "provider_error",
                                                    "title": "Provider error",
                                                    "text": error.to_string(),
                                                }),
                                            )));
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

    if let Err(error) = run_result {
        if let Some(sender) = started_tx.lock().ok().and_then(|mut guard| guard.take()) {
            drop(sender.send(Err(error.to_string())));
        }
        return Err(error.to_string());
    }

    drop(child.kill().await);
    Ok(())
}

fn push_session_update(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<AcpEvent>,
    update: SessionUpdate,
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
                    message_id: uuid::Uuid::new_v4().to_string(),
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
                    message_id: uuid::Uuid::new_v4().to_string(),
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
                    input: tool_call.raw_input.clone().unwrap_or(Value::Null),
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
            if let Some(status) = enum_value(&tool_call.status) {
                event_tx
                    .send(provider_info_event(
                        session_id.to_string(),
                        "codex",
                        json!({
                            "type": "tool_call_metadata",
                            "tool_call_id": tool_call.tool_call_id.to_string(),
                            "title": tool_call.title.clone(),
                            "tool_kind": enum_value(&tool_call.kind),
                            "status": status,
                            "locations": tool_call.locations.iter().map(|location| location.path.display().to_string()).collect::<Vec<_>>(),
                            "content": tool_call.content.clone(),
                            "raw_output": tool_call.raw_output.clone(),
                        }),
                    ))
                    .map_err(|_| "ACP event channel closed".to_string())?;
            }
        }
        SessionUpdate::ToolCallUpdate(update) => {
            event_tx
                .send(AcpEvent::ToolCallUpdate {
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: jiff::Timestamp::now().to_string(),
                    session_id: session_id.to_string(),
                    seq: 0,
                    tool_call_id: update.tool_call_id.to_string(),
                    output: tool_call_update_output(&update.fields),
                    status: update
                        .fields
                        .status
                        .as_ref()
                        .and_then(enum_value)
                        .unwrap_or_else(|| "updated".to_string()),
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

fn tool_call_update_output(fields: &agent_client_protocol::schema::ToolCallUpdateFields) -> Value {
    if let Some(raw_output) = fields.raw_output.clone() {
        return raw_output;
    }

    json!({
        "title": fields.title.clone(),
        "kind": fields.kind.as_ref().and_then(enum_value),
        "status": fields.status.as_ref().and_then(enum_value),
        "content": fields.content.clone(),
        "locations": fields.locations.as_ref().map(|locations| {
            locations
                .iter()
                .map(|location| location.path.display().to_string())
                .collect::<Vec<_>>()
        }),
        "raw_input": fields.raw_input.clone(),
    })
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
