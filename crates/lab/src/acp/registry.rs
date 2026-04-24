use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast, mpsc};

use crate::dispatch::error::ToolError;

use super::persistence::JsonFileAcpPersistence;
use super::runtime::{RuntimeHandle, codex_provider_health, launch_codex_runtime};
use super::types::{
    BridgeEvent, BridgeSessionStatus, BridgeSessionSummary, PendingBridgeEvent, ProviderHealth,
    StartSessionInput,
};

struct SessionRecord {
    summary: BridgeSessionSummary,
    events: Vec<BridgeEvent>,
    next_seq: u64,
    runtime: Option<RuntimeHandle>,
    event_tx: Option<mpsc::UnboundedSender<PendingBridgeEvent>>,
    broadcast_tx: broadcast::Sender<BridgeEvent>,
}

#[derive(Clone)]
pub struct AcpSessionRegistry {
    sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
    persistence: JsonFileAcpPersistence,
    default_cwd: String,
}

impl AcpSessionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let persistence = JsonFileAcpPersistence::new();
        let default_cwd = std::env::var("ACP_SESSION_CWD").unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
        let sessions = Self::load_sessions_sync(&persistence);
        Self {
            sessions: Arc::new(RwLock::new(sessions)),
            persistence,
            default_cwd,
        }
    }

    fn load_sessions_sync(persistence: &JsonFileAcpPersistence) -> HashMap<String, SessionRecord> {
        let mut map = HashMap::new();
        for summary in persistence.load_sessions_sync() {
            let events = persistence.load_events_sync(&summary.id);
            let next_seq = events.last().map(|event| event.seq + 1).unwrap_or(1);
            let (broadcast_tx, _) = broadcast::channel(256);
            map.insert(
                summary.id.clone(),
                SessionRecord {
                    summary,
                    events,
                    next_seq,
                    runtime: None,
                    event_tx: None,
                    broadcast_tx,
                },
            );
        }
        map
    }

    #[must_use]
    pub fn provider_health(&self) -> ProviderHealth {
        codex_provider_health()
    }

    pub async fn list_sessions(&self) -> Vec<BridgeSessionSummary> {
        let guard = self.sessions.read().await;
        let mut sessions: Vec<_> = guard.values().map(|record| record.summary.clone()).collect();
        sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        sessions
    }

    #[allow(dead_code)]
    pub async fn get_session(&self, session_id: &str) -> Option<BridgeSessionSummary> {
        let guard = self.sessions.read().await;
        guard.get(session_id).map(|record| record.summary.clone())
    }

    pub async fn create_session(
        self: &Arc<Self>,
        input: Option<StartSessionInput>,
    ) -> Result<BridgeSessionSummary, ToolError> {
        let input = input.unwrap_or(StartSessionInput {
            cwd: self.default_cwd.clone(),
            title: None,
        });
        let session_id = uuid::Uuid::new_v4().to_string();
        let created_at = jiff::Timestamp::now().to_string();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(256);
        self.spawn_event_forwarder(session_id.clone(), event_rx);

        let (runtime, started) = launch_codex_runtime(session_id.clone(), input.clone(), event_tx.clone())
            .await
            .map_err(|message| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message,
            })?;

        let summary = BridgeSessionSummary {
            id: session_id.clone(),
            provider_session_id: started.provider_session_id,
            provider: "codex".to_string(),
            title: input.title.unwrap_or_else(|| "New session".to_string()),
            cwd: input.cwd,
            created_at: created_at.clone(),
            updated_at: created_at,
            status: BridgeSessionStatus::Idle.as_str().to_string(),
            agent_name: started.agent_name,
            agent_version: started.agent_version,
            resumable: Some(true),
        };

        {
            let mut guard = self.sessions.write().await;
            guard.insert(
                session_id,
                SessionRecord {
                    summary: summary.clone(),
                    events: Vec::new(),
                    next_seq: 1,
                    runtime: Some(runtime),
                    event_tx: Some(event_tx),
                    broadcast_tx,
                },
            );
        }

        self.persist_all().await;
        Ok(summary)
    }

    pub async fn prompt_session(
        self: &Arc<Self>,
        session_id: &str,
        prompt: String,
    ) -> Result<(), ToolError> {
        let needs_reattach = {
            let mut guard = self.sessions.write().await;
            let Some(record) = guard.get_mut(session_id) else {
                return Err(not_found("unknown ACP session"));
            };
            record.summary.status = BridgeSessionStatus::Running.as_str().to_string();
            record.summary.updated_at = jiff::Timestamp::now().to_string();
            record.runtime.is_none()
        };

        if needs_reattach {
            self.reattach_runtime(session_id).await?;
        }

        let runtime = {
            let guard = self.sessions.read().await;
            guard
                .get(session_id)
                .and_then(|record| record.runtime.clone())
                .ok_or_else(|| internal("ACP runtime unavailable"))?
        };
        runtime.prompt(prompt).await.map_err(internal_message)?;
        self.persist_all().await;
        Ok(())
    }

    pub async fn cancel_session(&self, session_id: &str) -> Result<(), ToolError> {
        let runtime = {
            let mut guard = self.sessions.write().await;
            let Some(record) = guard.get_mut(session_id) else {
                return Err(not_found("unknown ACP session"));
            };
            record.summary.status = BridgeSessionStatus::Cancelled.as_str().to_string();
            record.summary.updated_at = jiff::Timestamp::now().to_string();
            record.runtime.clone()
        };
        if let Some(runtime) = runtime {
            runtime.cancel().await.map_err(internal_message)?;
        }
        self.persist_all().await;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        session_id: &str,
        since_seq: u64,
    ) -> Result<(Vec<BridgeEvent>, broadcast::Receiver<BridgeEvent>), ToolError> {
        let guard = self.sessions.read().await;
        let Some(record) = guard.get(session_id) else {
            return Err(not_found("unknown ACP session"));
        };
        let backlog = record
            .events
            .iter()
            .filter(|event| event.seq > since_seq)
            .cloned()
            .collect();
        Ok((backlog, record.broadcast_tx.subscribe()))
    }

    fn spawn_event_forwarder(self: &Arc<Self>, session_id: String, mut event_rx: mpsc::UnboundedReceiver<PendingBridgeEvent>) {
        let registry = Arc::clone(self);
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                drop(registry.push_event(session_id.clone(), event).await);
            }
        });
    }

    async fn reattach_runtime(self: &Arc<Self>, session_id: &str) -> Result<(), ToolError> {
        let (cwd, title) = {
            let guard = self.sessions.read().await;
            let Some(record) = guard.get(session_id) else {
                return Err(not_found("unknown ACP session"));
            };
            (record.summary.cwd.clone(), record.summary.title.clone())
        };
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        self.spawn_event_forwarder(session_id.to_string(), event_rx);

        let (runtime, started) = launch_codex_runtime(
            session_id.to_string(),
            StartSessionInput {
                cwd,
                title: Some(title),
            },
            event_tx.clone(),
        )
        .await
        .map_err(internal_message)?;

        {
            let mut guard = self.sessions.write().await;
            let Some(record) = guard.get_mut(session_id) else {
                return Err(not_found("unknown ACP session"));
            };
            record.runtime = Some(runtime);
            record.event_tx = Some(event_tx);
            record.summary.provider_session_id = started.provider_session_id;
            record.summary.agent_name = started.agent_name;
            record.summary.agent_version = started.agent_version;
            record.summary.updated_at = jiff::Timestamp::now().to_string();
            record.summary.resumable = Some(true);
        }

        self.push_event(
            session_id.to_string(),
            PendingBridgeEvent {
                title: Some("Session runtime recreated".to_string()),
                text: Some(
                    "The provider process was restarted and this session resumed with a new runtime."
                        .to_string(),
                ),
                status: Some("idle".to_string()),
                ..PendingBridgeEvent::new(session_id.to_string(), "codex", "status")
            },
        )
        .await?;

        Ok(())
    }

    async fn push_event(
        &self,
        session_id: String,
        event: PendingBridgeEvent,
    ) -> Result<(), ToolError> {
        let (cloned, events_snapshot, sender) = {
            let mut guard = self.sessions.write().await;
            let Some(record) = guard.get_mut(&session_id) else {
                return Err(not_found("unknown ACP session"));
            };
            let full = BridgeEvent {
                id: uuid::Uuid::new_v4().to_string(),
                seq: record.next_seq,
                session_id: event.session_id,
                provider: event.provider,
                kind: event.kind,
                created_at: event.created_at,
                role: event.role,
                message_id: event.message_id,
                text: event.text,
                title: event.title,
                status: event.status,
                tool_call_id: event.tool_call_id,
                tool_kind: event.tool_kind,
                raw_input: event.raw_input,
                raw_output: event.raw_output,
                tool_content: event.tool_content,
                locations: event.locations,
                plan: event.plan,
                permission_options: event.permission_options,
                permission_selection: event.permission_selection,
                session_info: event.session_info,
                usage: event.usage,
                commands: event.commands,
                current_mode: event.current_mode,
                config_update: event.config_update,
                prompt_stop_reason: event.prompt_stop_reason,
                raw: event.raw,
            };
            record.next_seq += 1;
            record.summary.updated_at = full.created_at.clone();
            if full.kind == "status" {
                if let Some(status) = &full.status {
                    record.summary.status = status.clone();
                }
            }
            if full.kind == "session.info" {
                if let Some(title) = full
                    .session_info
                    .as_ref()
                    .and_then(|value| value.get("title"))
                    .and_then(|value| value.as_str())
                {
                    record.summary.title = title.to_string();
                }
            }
            if full.kind == "error" {
                record.summary.status = BridgeSessionStatus::Failed.as_str().to_string();
            }
            record.events.push(full.clone());
            if record.events.len() > 500 {
                let extra = record.events.len() - 500;
                record.events.drain(0..extra);
            }
            let events_snapshot = record.events.clone();
            (full, events_snapshot, record.broadcast_tx.clone())
        };
        let sessions_snapshot = {
            let guard = self.sessions.read().await;
            guard.values().map(|entry| entry.summary.clone()).collect::<Vec<_>>()
        };

        drop(sender.send(cloned.clone()));
        drop(self.persistence.save_sessions(&sessions_snapshot).await);
        drop(self.persistence.save_events(&session_id, &events_snapshot).await);
        Ok(())
    }

    async fn persist_all(&self) {
        let (sessions, per_session) = {
            let guard = self.sessions.read().await;
            let sessions = guard.values().map(|record| record.summary.clone()).collect::<Vec<_>>();
            let per_session = guard
                .iter()
                .map(|(session_id, record)| (session_id.clone(), record.events.clone()))
                .collect::<Vec<_>>();
            (sessions, per_session)
        };
        drop(self.persistence.save_sessions(&sessions).await);
        for (session_id, events) in per_session {
            drop(self.persistence.save_events(&session_id, &events).await);
        }
    }
}

impl Default for AcpSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn internal(message: &str) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: message.to_string(),
    }
}

fn internal_message(message: String) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message,
    }
}

fn not_found(message: &str) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "not_found".to_string(),
        message: message.to_string(),
    }
}
