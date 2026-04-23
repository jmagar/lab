use std::path::Path;
use std::sync::{Arc, Mutex};

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
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::types::{
    BridgePermissionOption, PendingBridgeEvent, ProviderHealth, StartSessionInput,
    StartSessionResult,
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

pub async fn launch_codex_runtime(
    session_id: String,
    input: StartSessionInput,
    event_tx: mpsc::UnboundedSender<PendingBridgeEvent>,
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

pub fn codex_provider_health() -> ProviderHealth {
    let (command, args) = resolve_codex_launch();
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

    ProviderHealth {
        provider: "codex".to_string(),
        ready,
        command,
        args,
        message: if ready {
            "ACP Codex provider is available.".to_string()
        } else {
            "ACP Codex provider is unavailable. Set ACP_CODEX_COMMAND or ensure npx is on PATH."
                .to_string()
        },
    }
}

fn resolve_codex_launch() -> (String, Vec<String>) {
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
    event_tx: mpsc::UnboundedSender<PendingBridgeEvent>,
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
            drop(stderr_tx.send(PendingBridgeEvent {
                title: Some("codex-acp stderr".to_string()),
                text: Some(line),
                ..PendingBridgeEvent::new(stderr_session.clone(), "codex", "debug")
            }));
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

                    drop(event_tx.send(PendingBridgeEvent {
                        title: title.clone(),
                        tool_call_id: Some(args.tool_call.tool_call_id.to_string()),
                        permission_options: Some(
                            args.options
                                .iter()
                                .map(|option| BridgePermissionOption {
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
                        ),
                        status: Some("requested".to_string()),
                        raw: serde_json::to_value(&args).ok(),
                        ..PendingBridgeEvent::new(
                            session_id.clone(),
                            "codex",
                            "permission.requested",
                        )
                    }));

                    drop(event_tx.send(PendingBridgeEvent {
                        title,
                        tool_call_id: Some(args.tool_call.tool_call_id.to_string()),
                        permission_selection: selected.map(|option| option.option_id.to_string()),
                        status: Some("resolved".to_string()),
                        raw: serde_json::to_value(&args).ok(),
                        ..PendingBridgeEvent::new(
                            session_id.clone(),
                            "codex",
                            "permission.resolved",
                        )
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
                                drop(event_tx.send(PendingBridgeEvent {
                                    title: Some("Prompt started".to_string()),
                                    text: Some(prompt.clone()),
                                    status: Some("running".to_string()),
                                    ..PendingBridgeEvent::new(
                                        session_id.clone(),
                                        "codex",
                                        "status",
                                    )
                                }));

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
                                            drop(event_tx.send(PendingBridgeEvent {
                                                title: Some("Prompt completed".to_string()),
                                                status: Some(
                                                    if stop_reason == "cancelled" {
                                                        "cancelled".to_string()
                                                    } else {
                                                        "completed".to_string()
                                                    },
                                                ),
                                                prompt_stop_reason: Some(stop_reason),
                                                ..PendingBridgeEvent::new(
                                                    session_id.clone(),
                                                    "codex",
                                                    "status",
                                                )
                                            }));
                                            break;
                                        }
                                        Ok(_) => {
                                            drop(event_tx.send(PendingBridgeEvent {
                                                title: Some("Unhandled provider update".to_string()),
                                                status: Some("running".to_string()),
                                                ..PendingBridgeEvent::new(
                                                    session_id.clone(),
                                                    "codex",
                                                    "status",
                                                )
                                            }));
                                        }
                                        Err(error) => {
                                            drop(event_tx.send(PendingBridgeEvent {
                                                title: Some("Provider error".to_string()),
                                                text: Some(error.to_string()),
                                                status: Some("failed".to_string()),
                                                ..PendingBridgeEvent::new(
                                                    session_id.clone(),
                                                    "codex",
                                                    "error",
                                                )
                                            }));
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
    event_tx: &mpsc::UnboundedSender<PendingBridgeEvent>,
    update: SessionUpdate,
) -> Result<(), String> {
    match update {
        SessionUpdate::UserMessageChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(PendingBridgeEvent {
                    role: Some("user".to_string()),
                    text: Some(content_to_text(content)),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "message.chunk")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AgentMessageChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(PendingBridgeEvent {
                    role: Some("assistant".to_string()),
                    text: Some(content_to_text(content)),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "message.chunk")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AgentThoughtChunk(ContentChunk { content, .. }) => {
            event_tx
                .send(PendingBridgeEvent {
                    role: Some("thinking".to_string()),
                    text: Some(content_to_text(content)),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "message.chunk")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::ToolCall(tool_call) => {
            event_tx
                .send(PendingBridgeEvent {
                    title: Some(tool_call.title.clone()),
                    tool_call_id: Some(tool_call.tool_call_id.to_string()),
                    tool_kind: enum_value(&tool_call.kind),
                    status: enum_value(&tool_call.status),
                    raw_input: tool_call.raw_input.clone(),
                    raw_output: tool_call.raw_output.clone(),
                    tool_content: serde_json::to_value(&tool_call.content)
                        .ok()
                        .and_then(|value| value.as_array().cloned()),
                    locations: Some(
                        tool_call
                            .locations
                            .iter()
                            .map(|location| location.path.display().to_string())
                            .collect(),
                    ),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "tool.call")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::ToolCallUpdate(update) => {
            event_tx
                .send(PendingBridgeEvent {
                    title: update.fields.title.clone(),
                    tool_call_id: Some(update.tool_call_id.to_string()),
                    tool_kind: update.fields.kind.as_ref().and_then(enum_value),
                    status: update.fields.status.as_ref().and_then(enum_value),
                    raw_input: update.fields.raw_input.clone(),
                    raw_output: update.fields.raw_output.clone(),
                    tool_content: serde_json::to_value(&update.fields.content)
                        .ok()
                        .and_then(|value| value.as_array().cloned()),
                    locations: update.fields.locations.as_ref().map(|locations| {
                        locations
                            .iter()
                            .map(|location| location.path.display().to_string())
                            .collect()
                    }),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "tool.update")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::Plan(plan) => {
            event_tx
                .send(PendingBridgeEvent {
                    title: Some("Execution plan updated".to_string()),
                    plan: serde_json::to_value(&plan)
                        .ok()
                        .and_then(|value| value.get("entries").and_then(|entries| entries.as_array().cloned())),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "plan")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
        SessionUpdate::AvailableCommandsUpdate(update) => {
            event_tx
                .send(PendingBridgeEvent {
                    title: Some("Available commands updated".to_string()),
                    commands: serde_json::to_value(&update)
                        .ok()
                        .and_then(|value| value.get("commands").and_then(|commands| commands.as_array().cloned())),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "commands")
                })
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
                .send(PendingBridgeEvent {
                    title: Some("Unhandled session update".to_string()),
                    raw: serde_json::to_value(&other).ok(),
                    ..PendingBridgeEvent::new(session_id.to_string(), "codex", "debug")
                })
                .map_err(|_| "ACP event channel closed".to_string())?;
        }
    }

    Ok(())
}

fn emit_current_mode(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<PendingBridgeEvent>,
    update: CurrentModeUpdate,
) -> Result<(), String> {
    event_tx
        .send(PendingBridgeEvent {
            title: Some("Agent mode updated".to_string()),
            current_mode: serde_json::to_value(&update).ok(),
            ..PendingBridgeEvent::new(session_id.to_string(), "codex", "mode")
        })
        .map_err(|_| "ACP event channel closed".to_string())
}

fn emit_config_update(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<PendingBridgeEvent>,
    update: ConfigOptionUpdate,
) -> Result<(), String> {
    event_tx
        .send(PendingBridgeEvent {
            title: Some("Configuration options updated".to_string()),
            config_update: serde_json::to_value(&update).ok(),
            ..PendingBridgeEvent::new(session_id.to_string(), "codex", "config")
        })
        .map_err(|_| "ACP event channel closed".to_string())
}

fn emit_session_info(
    session_id: &str,
    event_tx: &mpsc::UnboundedSender<PendingBridgeEvent>,
    update: SessionInfoUpdate,
) -> Result<(), String> {
    event_tx
        .send(PendingBridgeEvent {
            title: Some("Session info updated".to_string()),
            session_info: serde_json::to_value(&update).ok(),
            ..PendingBridgeEvent::new(session_id.to_string(), "codex", "session.info")
        })
        .map_err(|_| "ACP event channel closed".to_string())
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
