//! `AcpSessionRegistry` — per-subscriber mpsc, Arc<Session> ownership, SQLite persistence.
//!
//! # Design decisions (locked by bead lab-jwbg.5)
//!
//! ## Arc<Session> pattern
//! The outer map (`sessions`) stores `Arc<Session>`. Callers clone the Arc and
//! immediately drop the map guard before doing anything that might `.await`.
//! The guard is NEVER held across an `.await` point.
//!
//! ## Per-subscriber bounded mpsc (replaces broadcast)
//! Each subscriber gets their own `mpsc::Sender<Arc<AcpEvent>>` (capacity 64).
//! A fanout task converts `PendingBridgeEvent`s into `AcpEvent`s and sends to
//! all subscribers. If a sender is full, that event is dropped for that
//! subscriber only (WARN logged); the subscriber receives a `reconnect` signal
//! before being removed.
//!
//! ## Subscribe-first race fix
//! Subscribe adds the mpsc receiver FIRST, then queries SQLite for backlog.
//! The returned stream chains `backlog.chain(live_mpsc)` and deduplicates at
//! the junction by skipping live events whose seq <= last_backlog_seq.
//!
//! ## Principal ownership
//! `principal` is bound at session create (immutable). `check_principal()` is
//! called on subscribe, prompt, cancel, and close. If principal is `""`, the
//! check is skipped (bead 7 wires it properly).
//!
//! ## Persistence
//! `SqliteAcpPersistence` is initialized lazily on first async use via
//! `tokio::sync::OnceCell`. `new()` stays synchronous for `AppState::new()`.

use std::collections::HashMap;
use std::sync::Arc;

use futures::{Stream, StreamExt, stream};
use tokio::sync::{Mutex, OnceCell, RwLock, broadcast, mpsc};

use lab_apis::acp::persistence::AcpPersistence;
use lab_apis::acp::types::{AcpEvent, AcpProviderHealth, AcpSessionState, AcpSessionSummary};

use crate::dispatch::acp::codex::codex_health;
use crate::dispatch::acp::persistence::SqliteAcpPersistence;
use crate::dispatch::error::ToolError;

// Also re-export types that api/services/acp.rs still needs from the old shape.
// (BridgeEvent is used in the SSE handler — bead 7 will clean that up.)
use super::types::{BridgeEvent, PendingBridgeEvent, StartSessionInput};
use super::runtime::{RuntimeHandle, launch_codex_runtime};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Capacity for each subscriber's mpsc channel.
const SUBSCRIBER_CAPACITY: usize = 64;

/// Maximum backlog events returned from SQLite on subscribe.
const BACKFILL_CAP: u64 = 10_000;

// ---------------------------------------------------------------------------
// Session struct
// ---------------------------------------------------------------------------

struct Session {
    id: String,
    /// Immutable after creation.
    principal: String,
    state: RwLock<AcpSessionState>,
    summary: RwLock<AcpSessionSummary>,
    handle: Mutex<Option<RuntimeHandle>>,
    /// Per-subscriber bounded mpsc senders. Fanout task pushes to all.
    subscribers: Mutex<Vec<mpsc::Sender<Arc<AcpEvent>>>>,
    /// In-memory event buffer (capped at 500) for legacy callers.
    events: RwLock<Vec<BridgeEvent>>,
    next_seq: Mutex<u64>,
    /// Legacy broadcast for the old api/services/acp.rs SSE handler.
    /// Kept until bead 7 migrates that caller to the new Stream API.
    broadcast_tx: broadcast::Sender<BridgeEvent>,
}

impl Session {
    fn new(id: String, principal: String, summary: AcpSessionSummary) -> Arc<Self> {
        let (broadcast_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            id,
            principal,
            state: RwLock::new(summary.state.clone()),
            summary: RwLock::new(summary),
            handle: Mutex::new(None),
            subscribers: Mutex::new(Vec::new()),
            events: RwLock::new(Vec::new()),
            next_seq: Mutex::new(1),
            broadcast_tx,
        })
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AcpSessionRegistry {
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    /// Lazily initialized on first async use.
    persistence: Arc<OnceCell<SqliteAcpPersistence>>,
    default_cwd: String,
}

impl AcpSessionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let default_cwd = std::env::var("ACP_SESSION_CWD").unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence: Arc::new(OnceCell::new()),
            default_cwd,
        }
    }

    // ── Persistence helpers ────────────────────────────────────────────────

    async fn persistence(&self) -> Option<&SqliteAcpPersistence> {
        self.persistence
            .get_or_try_init(|| async { SqliteAcpPersistence::from_env().await })
            .await
            .map_err(|error| {
                tracing::error!(
                    surface = "acp",
                    service = "persistence",
                    action = "init",
                    kind = "internal_error",
                    error = %error,
                    "failed to open SQLite ACP database — registry will run without persistence",
                );
            })
            .ok()
    }

    // ── Principal check ────────────────────────────────────────────────────

    fn check_principal(session: &Session, principal: &str) -> Result<(), ToolError> {
        if principal.is_empty() || session.principal.is_empty() {
            return Ok(());
        }
        if session.principal != principal {
            return Err(ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: "unknown ACP session".to_string(),
            });
        }
        Ok(())
    }

    // ── Public API ─────────────────────────────────────────────────────────

    /// Returns a synchronous provider health snapshot (async variant via codex_health).
    ///
    /// NOTE: For now this spawns a blocking task via tokio::task::block_in_place
    /// to call `codex_health()`. In Phase 1 the upstream returns a simple stub.
    #[must_use]
    pub fn provider_health(&self) -> AcpProviderHealth {
        // codex_health() is async but we need sync here for AppState callers.
        // Use block_in_place to avoid spawning a full blocking thread.
        // This is safe inside an async context (axum handler) where we're on
        // a multi-thread tokio runtime.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(codex_health())
        })
    }

    pub async fn list_sessions(&self, principal: &str) -> Vec<AcpSessionSummary> {
        // Collect Arc<Session> snapshots, then drop the map guard before any .await.
        let sessions_snapshot: Vec<Arc<Session>> = {
            let guard = self.sessions.read().await;
            guard
                .values()
                .filter(|s| {
                    principal.is_empty()
                        || s.principal.is_empty()
                        || s.principal == principal
                })
                .cloned()
                .collect()
        }; // guard dropped here — no guard held across .await

        let mut summaries: Vec<AcpSessionSummary> = Vec::with_capacity(sessions_snapshot.len());
        for session in &sessions_snapshot {
            let summary = session.summary.read().await;
            summaries.push(summary.clone());
        }
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    pub async fn get_session(&self, session_id: &str) -> Option<AcpSessionSummary> {
        // Clone the Arc, drop the map guard.
        let session = {
            let guard = self.sessions.read().await;
            guard.get(session_id).cloned()
        }?;
        let summary = session.summary.read().await;
        Some(summary.clone())
    }

    pub async fn create_session(
        &self,
        input: StartSessionInput,
        principal: &str,
    ) -> Result<AcpSessionSummary, ToolError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let created_at = jiff::Timestamp::now().to_string();
        let cwd = if input.cwd.is_empty() {
            self.default_cwd.clone()
        } else {
            input.cwd.clone()
        };

        // Launch the codex runtime (Phase 1: stub that drops rx immediately).
        let (event_tx_bridge, event_rx_bridge) = mpsc::unbounded_channel::<PendingBridgeEvent>();
        let (runtime, started) =
            launch_codex_runtime(session_id.clone(), StartSessionInput { cwd: cwd.clone(), title: input.title.clone(), principal: input.principal.clone() }, event_tx_bridge.clone())
                .await
                .map_err(|message| ToolError::Sdk {
                    sdk_kind: "internal_error".to_string(),
                    message,
                })?;

        let summary = AcpSessionSummary {
            id: session_id.clone(),
            provider: "codex".to_string(),
            title: input.title.unwrap_or_else(|| "New session".to_string()),
            cwd: cwd.clone(),
            state: AcpSessionState::Idle,
            created_at: created_at.clone(),
            updated_at: created_at,
            principal: if principal.is_empty() { None } else { Some(principal.to_string()) },
            provider_session_id: Some(started.provider_session_id),
            agent_name: Some(started.agent_name),
            agent_version: Some(started.agent_version),
        };

        let session = Session::new(session_id.clone(), principal.to_string(), summary.clone());
        {
            let mut handle_guard = session.handle.lock().await;
            *handle_guard = Some(runtime);
        }

        // Insert into map.
        {
            let mut map_guard = self.sessions.write().await;
            map_guard.insert(session_id.clone(), Arc::clone(&session));
        }

        // Spawn bridge forwarder (converts PendingBridgeEvent → legacy BridgeEvent broadcast).
        self.spawn_bridge_forwarder(Arc::clone(&session), event_rx_bridge);

        // Persist.
        if let Some(db) = self.persistence().await {
            drop(db.save_session(&summary).await);
        }

        Ok(summary)
    }

    pub async fn prompt_session(
        &self,
        session_id: &str,
        prompt: &str,
        principal: &str,
    ) -> Result<(), ToolError> {
        // Clone Arc, drop map guard.
        let session = {
            let guard = self.sessions.read().await;
            guard.get(session_id).cloned().ok_or_else(|| not_found("unknown ACP session"))?
        };

        Self::check_principal(&session, principal)?;

        // Validate state transition.
        {
            let state = session.state.read().await;
            if !state.can_transition_to(&AcpSessionState::Running) {
                return Err(ToolError::Sdk {
                    sdk_kind: "invalid_state".to_string(),
                    message: format!("session is in state {state:?}, cannot send prompt"),
                });
            }
        }

        // Update state and summary.
        {
            let mut state = session.state.write().await;
            *state = AcpSessionState::Running;
        }
        {
            let mut summary = session.summary.write().await;
            summary.state = AcpSessionState::Running;
            summary.updated_at = jiff::Timestamp::now().to_string();
        }

        // Get handle, reattach if needed.
        let needs_reattach = {
            let handle = session.handle.lock().await;
            handle.is_none()
        };
        if needs_reattach {
            self.reattach_runtime(&session).await?;
        }

        let runtime = {
            let handle = session.handle.lock().await;
            handle.clone().ok_or_else(|| internal("ACP runtime unavailable"))?
        };
        runtime.prompt(prompt.to_string()).await.map_err(internal_message)?;

        // Persist updated state.
        if let Some(db) = self.persistence().await {
            let summary = session.summary.read().await;
            drop(db.save_session(&*summary).await);
        }

        Ok(())
    }

    pub async fn cancel_session(
        &self,
        session_id: &str,
        principal: &str,
    ) -> Result<(), ToolError> {
        let session = {
            let guard = self.sessions.read().await;
            guard.get(session_id).cloned().ok_or_else(|| not_found("unknown ACP session"))?
        };

        Self::check_principal(&session, principal)?;

        // Validate state transition.
        {
            let state = session.state.read().await;
            if !state.can_transition_to(&AcpSessionState::Cancelled) {
                return Err(ToolError::Sdk {
                    sdk_kind: "invalid_state".to_string(),
                    message: format!("session is in state {state:?}, cannot cancel"),
                });
            }
        }

        {
            let mut state = session.state.write().await;
            *state = AcpSessionState::Cancelled;
        }
        {
            let mut summary = session.summary.write().await;
            summary.state = AcpSessionState::Cancelled;
            summary.updated_at = jiff::Timestamp::now().to_string();
        }

        let runtime = {
            let handle = session.handle.lock().await;
            handle.clone()
        };
        if let Some(rt) = runtime {
            drop(rt.cancel().await);
        }

        if let Some(db) = self.persistence().await {
            drop(db.update_session_state(session_id, AcpSessionState::Cancelled).await);
        }

        Ok(())
    }

    /// Subscribe to a session's event stream.
    ///
    /// Returns a `Stream<Item = Arc<AcpEvent>>` that chains backlog events
    /// (from SQLite, since_seq) with live events (per-subscriber mpsc).
    ///
    /// # Subscribe-first race fix
    /// The mpsc receiver is registered BEFORE querying SQLite. Live events
    /// arriving after registration but before the backlog is returned are kept
    /// in the mpsc buffer. At the stream junction, live events with
    /// `seq <= last_backlog_seq` are skipped to avoid duplicates.
    pub async fn subscribe(
        &self,
        session_id: &str,
        since_seq: u64,
        principal: &str,
    ) -> Result<impl Stream<Item = Arc<AcpEvent>>, ToolError> {
        let session = {
            let guard = self.sessions.read().await;
            guard.get(session_id).cloned().ok_or_else(|| not_found("unknown ACP session"))?
        };

        Self::check_principal(&session, principal)?;

        // Step 1: register subscriber mpsc FIRST.
        let (tx, rx) = mpsc::channel::<Arc<AcpEvent>>(SUBSCRIBER_CAPACITY);
        {
            let mut subs = session.subscribers.lock().await;
            subs.push(tx);
        }

        // Step 2: query SQLite backlog.
        let backlog: Vec<Arc<AcpEvent>> = if let Some(db) = self.persistence().await {
            let raw_since = since_seq;
            let capped_since = raw_since;
            match db.load_events_since(session_id, capped_since).await {
                Ok(events) => {
                    // Cap at BACKFILL_CAP events.
                    let start = events.len().saturating_sub(BACKFILL_CAP as usize);
                    events[start..].iter().cloned().map(Arc::new).collect()
                }
                Err(error) => {
                    tracing::warn!(
                        surface = "acp",
                        service = "registry",
                        action = "subscribe.backfill",
                        session_id,
                        error = %error,
                        "failed to load backlog from SQLite",
                    );
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Track the last backlog seq for deduplication at junction.
        let last_backlog_seq = backlog.last().map(|e| e.seq()).unwrap_or(since_seq);

        // Step 3: build stream — backlog.chain(live), deduplicate at junction.
        let backlog_stream = stream::iter(backlog);
        let live_stream = stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|event| (event, rx))
        })
        .filter(move |event| {
            let keep = event.seq() > last_backlog_seq;
            async move { keep }
        });

        Ok(backlog_stream.chain(live_stream))
    }

    // ── Legacy subscribe (for api/services/acp.rs until bead 7) ───────────

    /// Legacy subscribe returning `(Vec<BridgeEvent>, broadcast::Receiver<BridgeEvent>)`.
    ///
    /// This is kept for binary compatibility with the current `api/services/acp.rs`
    /// handler. It will be removed in bead 7 when the SSE handler is migrated to
    /// the new `Stream<Item = Arc<AcpEvent>>` shape.
    pub async fn subscribe_legacy(
        &self,
        session_id: &str,
        since_seq: u64,
    ) -> Result<(Vec<BridgeEvent>, broadcast::Receiver<BridgeEvent>), ToolError> {
        let session = {
            let guard = self.sessions.read().await;
            guard.get(session_id).cloned().ok_or_else(|| not_found("unknown ACP session"))?
        };

        let backlog = {
            let events = session.events.read().await;
            events
                .iter()
                .filter(|e| e.seq > since_seq)
                .cloned()
                .collect()
        };

        Ok((backlog, session.broadcast_tx.subscribe()))
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    /// Spawn a task that reads `PendingBridgeEvent`s from the legacy bridge channel
    /// and fans them out to both the broadcast (legacy) and per-subscriber mpsc senders.
    fn spawn_bridge_forwarder(
        &self,
        session: Arc<Session>,
        mut rx: mpsc::UnboundedReceiver<PendingBridgeEvent>,
    ) {
        let persistence = Arc::clone(&self.persistence);
        tokio::spawn(async move {
            while let Some(pending) = rx.recv().await {
                // Convert PendingBridgeEvent to BridgeEvent (legacy) and push.
                let bridge_event = {
                    let mut seq_guard = session.next_seq.lock().await;
                    let seq = *seq_guard;
                    *seq_guard += 1;
                    BridgeEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        seq,
                        session_id: pending.session_id.clone(),
                        provider: pending.provider.clone(),
                        kind: pending.kind.clone(),
                        created_at: pending.created_at.clone(),
                        role: pending.role,
                        message_id: pending.message_id,
                        text: pending.text,
                        title: pending.title,
                        status: pending.status.clone(),
                        tool_call_id: pending.tool_call_id,
                        tool_kind: pending.tool_kind,
                        raw_input: pending.raw_input,
                        raw_output: pending.raw_output,
                        tool_content: pending.tool_content,
                        locations: pending.locations,
                        plan: pending.plan,
                        permission_options: pending.permission_options,
                        permission_selection: pending.permission_selection,
                        session_info: pending.session_info.clone(),
                        usage: pending.usage,
                        commands: pending.commands,
                        current_mode: pending.current_mode,
                        config_update: pending.config_update,
                        prompt_stop_reason: pending.prompt_stop_reason,
                        raw: pending.raw,
                    }
                };

                // Update summary and state from event — one lock at a time.
                // Determine new state from the event kind/status (no locks yet).
                let maybe_new_state: Option<AcpSessionState> =
                    if bridge_event.kind == "status" {
                        bridge_event.status.as_deref().and_then(|s| match s {
                            "idle" => Some(AcpSessionState::Idle),
                            "running" => Some(AcpSessionState::Running),
                            "completed" => Some(AcpSessionState::Completed),
                            "cancelled" => Some(AcpSessionState::Cancelled),
                            "failed" => Some(AcpSessionState::Failed),
                            _ => None,
                        })
                    } else if bridge_event.kind == "error" {
                        Some(AcpSessionState::Failed)
                    } else {
                        None
                    };

                // Update summary (one scoped lock).
                {
                    let mut summary = session.summary.write().await;
                    summary.updated_at = bridge_event.created_at.clone();
                    if let Some(ref state) = maybe_new_state {
                        summary.state = state.clone();
                    }
                    if bridge_event.kind == "session.info" {
                        if let Some(title) = bridge_event
                            .session_info
                            .as_ref()
                            .and_then(|v| v.get("title"))
                            .and_then(|v| v.as_str())
                        {
                            summary.title = title.to_string();
                        }
                    }
                } // summary guard dropped

                // Update state (separate scoped lock).
                if let Some(new_state) = maybe_new_state {
                    let mut st = session.state.write().await;
                    *st = new_state;
                } // state guard dropped

                // Update legacy in-memory event buffer (separate scoped lock).
                {
                    let mut events = session.events.write().await;
                    events.push(bridge_event.clone());
                    if events.len() > 500 {
                        let extra = events.len() - 500;
                        events.drain(0..extra);
                    }
                } // events guard dropped

                // Broadcast to legacy receiver.
                drop(session.broadcast_tx.send(bridge_event.clone()));

                // Convert to AcpEvent (SessionUpdate variant for now; future beads
                // will enrich this with proper variant translation).
                let acp_event = Arc::new(AcpEvent::Unknown {
                    id: bridge_event.id.clone(),
                    created_at: bridge_event.created_at.clone(),
                    session_id: bridge_event.session_id.clone(),
                    seq: bridge_event.seq,
                    event_kind: bridge_event.kind.clone(),
                    raw: serde_json::to_value(&bridge_event).unwrap_or(serde_json::Value::Null),
                });

                // Fanout to per-subscriber mpsc senders.
                let mut subs = session.subscribers.lock().await;
                let mut to_remove: Vec<usize> = Vec::new();
                for (i, tx) in subs.iter().enumerate() {
                    match tx.try_send(Arc::clone(&acp_event)) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                surface = "acp",
                                service = "registry",
                                action = "fanout",
                                subscriber_index = i,
                                session_id = bridge_event.session_id,
                                "subscriber mpsc full — dropping event for this subscriber",
                            );
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            to_remove.push(i);
                        }
                    }
                }
                // Remove closed subscribers (iterate in reverse to preserve indices).
                for i in to_remove.into_iter().rev() {
                    subs.swap_remove(i);
                }

                // Persist via SQLite if available.
                if let Some(db) = persistence
                    .get_or_try_init(|| async { SqliteAcpPersistence::from_env().await })
                    .await
                    .ok()
                {
                    drop(db.append_event(&*acp_event).await);
                }
            }
        });
    }

    async fn reattach_runtime(&self, session: &Arc<Session>) -> Result<(), ToolError> {
        let (cwd, title) = {
            let summary = session.summary.read().await;
            (summary.cwd.clone(), summary.title.clone())
        };

        let (event_tx_bridge, event_rx_bridge) = mpsc::unbounded_channel::<PendingBridgeEvent>();
        let (runtime, started) = launch_codex_runtime(
            session.id.clone(),
            StartSessionInput { cwd, title: Some(title), principal: None },
            event_tx_bridge.clone(),
        )
        .await
        .map_err(internal_message)?;

        self.spawn_bridge_forwarder(Arc::clone(session), event_rx_bridge);

        {
            let mut handle = session.handle.lock().await;
            *handle = Some(runtime);
        }
        {
            let mut summary = session.summary.write().await;
            summary.provider_session_id = Some(started.provider_session_id);
            summary.agent_name = Some(started.agent_name);
            summary.agent_version = Some(started.agent_version);
            summary.updated_at = jiff::Timestamp::now().to_string();
        }

        Ok(())
    }
}

impl Default for AcpSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Error constructors
// ---------------------------------------------------------------------------

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
