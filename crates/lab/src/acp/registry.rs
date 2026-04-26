//! `AcpSessionRegistry` — per-subscriber mpsc, Arc<Session> ownership, SQLite persistence.
//!
//! # Design decisions (locked by bead lab-jwbg.5)
//!
//! ## Arc<Session> pattern
//! The outer map (`sessions`) stores `Arc<Session>`. Callers clone the Arc and
//! immediately drop the map guard before doing anything that might `.await`.
//! The guard is NEVER held across an `.await` point.
//!
//! ## Per-subscriber bounded mpsc
//! Each subscriber gets their own `mpsc::Sender<Arc<AcpEvent>>` (capacity 64).
//! A fanout task sends typed `AcpEvent`s to all subscribers. If a sender is
//! full, that event is dropped for that
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
use tokio::sync::{Mutex, OnceCell, RwLock, mpsc};

use lab_apis::acp::persistence::AcpPersistence;
use lab_apis::acp::types::{AcpEvent, AcpProviderHealth, AcpSessionState, AcpSessionSummary};

use crate::dispatch::acp::persistence::SqliteAcpPersistence;
use crate::dispatch::error::ToolError;

use super::runtime::{
    RuntimeHandle, launch_codex_runtime, normalize_provider_id, provider_healths,
};
use super::types::{
    StartSessionInput, event_created_at, session_title_from_event, stamp_event_sequence,
};

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
    /// In-memory typed event buffer (capped at 500) when persistence is unavailable.
    events: RwLock<Vec<AcpEvent>>,
    next_seq: Mutex<u64>,
}

impl Session {
    fn new(id: String, principal: String, summary: AcpSessionSummary) -> Arc<Self> {
        Arc::new(Self {
            id,
            principal,
            state: RwLock::new(summary.state.clone()),
            summary: RwLock::new(summary),
            handle: Mutex::new(None),
            subscribers: Mutex::new(Vec::new()),
            events: RwLock::new(Vec::new()),
            next_seq: Mutex::new(1),
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

    // ── Session lookup ─────────────────────────────────────────────────────

    async fn get_session_arc(&self, session_id: &str) -> Result<Arc<Session>, ToolError> {
        let guard = self.sessions.read().await;
        guard
            .get(session_id)
            .cloned()
            .ok_or_else(|| not_found("unknown ACP session"))
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

    #[must_use]
    pub fn provider_healths(&self) -> Vec<AcpProviderHealth> {
        provider_healths()
    }

    pub async fn list_sessions(&self, principal: &str) -> Vec<AcpSessionSummary> {
        // Collect Arc<Session> snapshots, then drop the map guard before any .await.
        let sessions_snapshot: Vec<Arc<Session>> = {
            let guard = self.sessions.read().await;
            guard
                .values()
                .filter(|s| {
                    principal.is_empty() || s.principal.is_empty() || s.principal == principal
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
        let (event_tx, event_rx) = mpsc::unbounded_channel::<AcpEvent>();
        let (runtime, started) = launch_codex_runtime(
            session_id.clone(),
            StartSessionInput {
                provider: input.provider.clone(),
                cwd: cwd.clone(),
                title: input.title.clone(),
                principal: input.principal.clone(),
            },
            event_tx.clone(),
        )
        .await
        .map_err(|message| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message,
        })?;

        let provider = normalize_provider_id(input.provider.as_deref());
        let summary = AcpSessionSummary {
            id: session_id.clone(),
            provider,
            title: input.title.unwrap_or_else(|| "New session".to_string()),
            cwd: cwd.clone(),
            state: AcpSessionState::Idle,
            created_at: created_at.clone(),
            updated_at: created_at,
            principal: if principal.is_empty() {
                None
            } else {
                Some(principal.to_string())
            },
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

        // Spawn typed ACP event fanout.
        self.spawn_event_forwarder(Arc::clone(&session), event_rx);

        // Persist.
        if let Some(db) = self.persistence().await {
            if let Err(error) = db.save_session(&summary).await {
                tracing::warn!(
                    surface = "acp",
                    service = "registry",
                    action = "session.save",
                    session_id = %summary.id,
                    error = %error,
                    "failed to persist session summary",
                );
            }
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
        let session = self.get_session_arc(session_id).await?;

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
            handle
                .clone()
                .ok_or_else(|| internal("ACP runtime unavailable"))?
        };
        runtime
            .prompt(prompt.to_string())
            .await
            .map_err(internal_message)?;

        // Persist updated state.
        if let Some(db) = self.persistence().await {
            let summary = session.summary.read().await;
            if let Err(error) = db.save_session(&summary).await {
                tracing::warn!(
                    surface = "acp",
                    service = "registry",
                    action = "session.save",
                    session_id,
                    error = %error,
                    "failed to persist session summary after prompt",
                );
            }
        }

        Ok(())
    }

    pub async fn cancel_session(&self, session_id: &str, principal: &str) -> Result<(), ToolError> {
        let session = self.get_session_arc(session_id).await?;

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
            if let Err(error) = db
                .update_session_state(session_id, AcpSessionState::Cancelled)
                .await
            {
                tracing::warn!(
                    surface = "acp",
                    service = "registry",
                    action = "session.state",
                    session_id,
                    error = %error,
                    "failed to persist cancelled session state",
                );
            }
        }

        Ok(())
    }

    pub async fn close_session(&self, session_id: &str, principal: &str) -> Result<(), ToolError> {
        let session = self.get_session_arc(session_id).await?;

        Self::check_principal(&session, principal)?;

        {
            let mut state = session.state.write().await;
            *state = AcpSessionState::Closed;
        }
        {
            let mut summary = session.summary.write().await;
            summary.state = AcpSessionState::Closed;
            summary.updated_at = jiff::Timestamp::now().to_string();
        }

        // Abort runtime if running.
        let runtime = {
            let handle = session.handle.lock().await;
            handle.clone()
        };
        if let Some(rt) = runtime {
            drop(rt.cancel().await);
        }

        if let Some(db) = self.persistence().await {
            if let Err(error) = db
                .update_session_state(session_id, AcpSessionState::Closed)
                .await
            {
                tracing::warn!(
                    surface = "acp",
                    service = "registry",
                    action = "session.state",
                    session_id,
                    error = %error,
                    "failed to persist closed session state",
                );
            }
        }

        Ok(())
    }

    /// Get stored events for a session since a given sequence number.
    pub async fn get_events_since(
        &self,
        session_id: &str,
        since_seq: u64,
        principal: &str,
    ) -> Result<Vec<AcpEvent>, ToolError> {
        let session = self.get_session_arc(session_id).await?;

        Self::check_principal(&session, principal)?;

        if let Some(db) = self.persistence().await {
            match db.load_events_since(session_id, since_seq).await {
                Ok(events) => return Ok(events),
                Err(error) => {
                    tracing::warn!(
                        surface = "acp",
                        service = "registry",
                        action = "events.load",
                        session_id,
                        since_seq,
                        error = %error,
                        "failed to load persisted events, falling back to in-memory transcript",
                    );
                }
            }
        }

        Ok(load_in_memory_events(&session, since_seq).await)
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
    ) -> Result<impl Stream<Item = Arc<AcpEvent>> + use<>, ToolError> {
        let session = self.get_session_arc(session_id).await?;

        Self::check_principal(&session, principal)?;

        // Step 1: register subscriber mpsc FIRST.
        let (tx, rx) = mpsc::channel::<Arc<AcpEvent>>(SUBSCRIBER_CAPACITY);
        {
            let mut subs = session.subscribers.lock().await;
            subs.push(tx);
        }

        // Step 2: query SQLite backlog.
        let backlog: Vec<Arc<AcpEvent>> = if let Some(db) = self.persistence().await {
            match db.load_events_since(session_id, since_seq).await {
                Ok(events) => {
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
                        "failed to load backlog from SQLite, using in-memory transcript",
                    );
                    load_in_memory_events(&session, since_seq)
                        .await
                        .into_iter()
                        .map(Arc::new)
                        .collect()
                }
            }
        } else {
            load_in_memory_events(&session, since_seq)
                .await
                .into_iter()
                .map(Arc::new)
                .collect()
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

    // ── Internal helpers ───────────────────────────────────────────────────

    /// Spawn a task that reads typed ACP events from the runtime channel and
    /// fans them out to typed subscriber streams.
    fn spawn_event_forwarder(
        &self,
        session: Arc<Session>,
        mut rx: mpsc::UnboundedReceiver<AcpEvent>,
    ) {
        let registry = self.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let event = next_session_event(&session, event).await;
                persist_session_event(&registry, &event).await;
                apply_session_event(&session, &event).await;

                let dropped = fanout_event(&session, Arc::new(event.clone())).await;
                if dropped > 0 {
                    let marker = next_session_event(
                        &session,
                        AcpEvent::ProviderInfo {
                            id: uuid::Uuid::new_v4().to_string(),
                            created_at: jiff::Timestamp::now().to_string(),
                            session_id: session.id.clone(),
                            seq: 0,
                            provider: "lab".to_string(),
                            raw: serde_json::json!({
                                "type": "subscriber_backpressure",
                                "dropped_subscribers": dropped,
                                "after_seq": event.seq(),
                            }),
                        },
                    )
                    .await;

                    tracing::warn!(
                        surface = "acp",
                        service = "registry",
                        action = "fanout.backpressure",
                        session_id = %session.id,
                        dropped_subscribers = dropped,
                        after_seq = event.seq(),
                        "subscriber backpressure removed live subscribers; replay required from transcript",
                    );

                    persist_session_event(&registry, &marker).await;
                    apply_session_event(&session, &marker).await;
                    let _ = fanout_event(&session, Arc::new(marker)).await;
                }
            }
        });
    }

    async fn reattach_runtime(&self, session: &Arc<Session>) -> Result<(), ToolError> {
        let (provider, cwd, title) = {
            let summary = session.summary.read().await;
            (
                summary.provider.clone(),
                summary.cwd.clone(),
                summary.title.clone(),
            )
        };

        let (event_tx, event_rx) = mpsc::unbounded_channel::<AcpEvent>();
        let (runtime, started) = launch_codex_runtime(
            session.id.clone(),
            StartSessionInput {
                provider: Some(provider),
                cwd,
                title: Some(title),
                principal: None,
            },
            event_tx.clone(),
        )
        .await
        .map_err(internal_message)?;

        self.spawn_event_forwarder(Arc::clone(session), event_rx);

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

async fn load_in_memory_events(session: &Arc<Session>, since_seq: u64) -> Vec<AcpEvent> {
    let events = session.events.read().await;
    let filtered: Vec<AcpEvent> = events
        .iter()
        .filter(|event| event.seq() > since_seq)
        .cloned()
        .collect();
    let start = filtered.len().saturating_sub(BACKFILL_CAP as usize);
    filtered[start..].to_vec()
}

async fn next_session_event(session: &Arc<Session>, event: AcpEvent) -> AcpEvent {
    let mut seq_guard = session.next_seq.lock().await;
    let seq = *seq_guard;
    *seq_guard += 1;
    stamp_event_sequence(event, seq)
}

async fn apply_session_event(session: &Arc<Session>, event: &AcpEvent) {
    let maybe_new_state = session_state_from_event(event);

    {
        let mut summary = session.summary.write().await;
        summary.updated_at = event_created_at(event).to_string();
        if let Some(ref state) = maybe_new_state {
            summary.state = state.clone();
        }
        if let Some(title) = session_title_from_event(event) {
            summary.title = title;
        }
    }

    if let Some(new_state) = maybe_new_state {
        let mut state = session.state.write().await;
        *state = new_state;
    }

    {
        let mut events = session.events.write().await;
        events.push(event.clone());
        if events.len() > 500 {
            let extra = events.len() - 500;
            events.drain(0..extra);
        }
    }
}

fn session_state_from_event(event: &AcpEvent) -> Option<AcpSessionState> {
    match event {
        AcpEvent::SessionUpdate { state, .. } => Some(state.clone()),
        AcpEvent::PermissionRequest { .. } => Some(AcpSessionState::WaitingForPermission),
        AcpEvent::PermissionOutcome { granted, .. } => Some(if *granted {
            AcpSessionState::Running
        } else {
            AcpSessionState::Cancelled
        }),
        _ => None,
    }
}

async fn fanout_event(session: &Arc<Session>, event: Arc<AcpEvent>) -> usize {
    let mut subs = session.subscribers.lock().await;
    let mut to_remove: Vec<usize> = Vec::new();
    let mut dropped = 0usize;
    for (i, tx) in subs.iter().enumerate() {
        match tx.try_send(Arc::clone(&event)) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                dropped += 1;
                to_remove.push(i);
                tracing::warn!(
                    surface = "acp",
                    service = "registry",
                    action = "fanout",
                    subscriber_index = i,
                    session_id = event.session_id(),
                    seq = event.seq(),
                    "subscriber mpsc full — subscriber removed and must replay from transcript",
                );
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                to_remove.push(i);
            }
        }
    }
    for i in to_remove.into_iter().rev() {
        subs.swap_remove(i);
    }
    dropped
}

async fn persist_session_event(registry: &AcpSessionRegistry, event: &AcpEvent) {
    if let Some(db) = registry.persistence().await {
        if let Err(error) = db.append_event(event).await {
            tracing::warn!(
                surface = "acp",
                service = "registry",
                action = "event.persist",
                session_id = event.session_id(),
                seq = event.seq(),
                error = %error,
                "failed to persist typed acp event; replay is limited to in-memory history",
            );
        }

        if let Some(state) = session_state_from_event(event)
            && let Err(error) = db.update_session_state(event.session_id(), state).await
        {
            tracing::warn!(
                surface = "acp",
                service = "registry",
                action = "session.state.persist",
                session_id = event.session_id(),
                seq = event.seq(),
                error = %error,
                "failed to persist acp session state from event",
            );
        }
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
