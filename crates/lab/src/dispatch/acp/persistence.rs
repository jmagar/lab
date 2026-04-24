//! SQLite-backed ACP persistence.
//!
//! # Architecture
//!
//! Three connections are maintained:
//! - `write_conn` — dedicated write connection for session upserts and state
//!   updates (via `spawn_blocking`).
//! - `read_conn` — dedicated read-only connection; WAL mode allows concurrent
//!   reads without contending on `write_conn`.
//! - Writer task connection — privately owned by the background writer task
//!   that drains the mpsc channel and batches `acp_session_events` INSERTs
//!   (up to 64 events or 10 ms, whichever comes first).
//!
//! # Path security
//!
//! The `LAB_ACP_DB` path is validated to reject any component that is a
//! `ParentDir` (`..`). The soft-canonicalize crate is yanked; we do the
//! check manually with `std::path::Component`.
//!
//! # File permissions
//!
//! The database file is created with mode 0600 (owner read/write only) on
//! first open. Subsequent opens do not change permissions.
//!
//! # HMAC-signed permission outcomes
//!
//! `PermissionOutcome` events have their `granted` field signed with
//! HMAC-SHA256 before storage. The key is read from `LAB_ACP_HMAC_SECRET`
//! (auto-generated on first use and persisted to `~/.lab/.env`). This
//! prevents DB-write bypass attacks where an attacker could flip a `false`
//! grant to `true` in the raw SQLite file.
//!
//! # Payload redaction
//!
//! Before any event payload is written, fields named `token`, `api_key`,
//! `password`, `secret`, or `authorization` are replaced with `"[REDACTED]"`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::Value;
use sha2::Sha256;
use tokio::sync::mpsc;

use lab_apis::acp::error::{AcpError, PersistenceError};
use lab_apis::acp::persistence::AcpPersistence;
use lab_apis::acp::types::{AcpEvent, AcpSessionState, AcpSessionSummary};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum number of events to accumulate before flushing.
const BATCH_SIZE: usize = 64;

/// Maximum wait before flushing a partial batch.
const BATCH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);

// ── Types ─────────────────────────────────────────────────────────────────────

type HmacSha256 = Hmac<Sha256>;

/// Commands sent from the public API to the writer task.
enum PersistCmd {
    /// Event with pre-redacted, pre-serialized payload. The typed `AcpEvent`
    /// is kept for envelope-field accessors (id/session_id/seq/kind/created_at);
    /// `payload` is the JSON string written to the DB.
    AppendEvent(AcpEvent, String),
    /// Flush the current batch immediately and notify via the oneshot sender.
    /// Used for testing and graceful shutdown.
    #[allow(dead_code)]
    Flush(tokio::sync::oneshot::Sender<Result<(), String>>),
}

// ── SqliteAcpPersistence ──────────────────────────────────────────────────────

/// SQLite-backed implementation of `AcpPersistence`.
///
/// Clone is cheap — all state is behind `Arc`.
#[derive(Clone)]
pub struct SqliteAcpPersistence {
    /// Write connection for session upserts and state updates.
    write_conn: Arc<Mutex<Connection>>,
    /// Read-only connection; WAL mode allows concurrent reads.
    read_conn: Arc<Mutex<Connection>>,
    /// Channel to the background writer task (hot path for event appends).
    event_tx: mpsc::Sender<PersistCmd>,
    /// HMAC key for signing permission outcomes.
    hmac_key: Arc<Vec<u8>>,
}

impl SqliteAcpPersistence {
    /// Open (or create) the ACP database.
    ///
    /// `db_path` must not contain `..` components. The file is created with
    /// mode 0600 on first open.
    pub async fn open(db_path: PathBuf) -> Result<Self, AcpError> {
        crate::dispatch::helpers::reject_path_traversal(&db_path.to_string_lossy()).map_err(
            |_| {
                AcpError::Internal(format!(
                    "LAB_ACP_DB path must not contain `..` components: {}",
                    db_path.display()
                ))
            },
        )?;

        let hmac_key = Arc::new(load_or_generate_hmac_key());
        let path = db_path.clone();

        let (write_conn, read_conn, writer_task_conn) =
            tokio::task::spawn_blocking(move || -> Result<_, rusqlite::Error> {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        rusqlite::Error::InvalidPath(format!("create_dir_all: {e}").into())
                    })?;
                }

                // Create the file with 0600 perms if it doesn't exist.
                #[cfg(unix)]
                create_db_file_0600(&path);

                let rw_flags =
                    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;

                // Write connection — applies schema migrations.
                let wc = Connection::open_with_flags(&path, rw_flags)?;
                wc.busy_timeout(std::time::Duration::from_millis(5_000))?;
                apply_wal_pragmas(&wc)?;
                migrate(&wc)?;

                // Read connection — query_only after schema is ready.
                let rc = Connection::open_with_flags(&path, rw_flags)?;
                rc.busy_timeout(std::time::Duration::from_millis(5_000))?;
                apply_wal_pragmas(&rc)?;
                rc.pragma_update(None, "query_only", "true")?;

                // Writer task connection — owns the hot-path INSERTs.
                let tc = Connection::open_with_flags(&path, rw_flags)?;
                tc.busy_timeout(std::time::Duration::from_millis(5_000))?;
                apply_wal_pragmas(&tc)?;

                Ok((wc, rc, tc))
            })
            .await
            .map_err(|e| AcpError::Internal(format!("db open join: {e}")))?
            .map_err(|e| AcpError::Persistence(PersistenceError::Sqlite(e.to_string())))?;

        // Bounded channel — back-pressure if writer falls behind.
        let (event_tx, event_rx) = mpsc::channel::<PersistCmd>(4096);

        // Spawn the background writer task.
        let writer_conn = Arc::new(Mutex::new(writer_task_conn));
        tokio::spawn(writer_task(writer_conn, event_rx));

        Ok(Self {
            write_conn: Arc::new(Mutex::new(write_conn)),
            read_conn: Arc::new(Mutex::new(read_conn)),
            event_tx,
            hmac_key,
        })
    }

    /// Open using the path from `LAB_ACP_DB` (or a default under `~/.lab/`).
    pub async fn from_env() -> Result<Self, AcpError> {
        let path = resolve_db_path()?;
        Self::open(path).await
    }

    // ── Internal helpers ───────────────────────────────────────────────────────

    async fn blocking_write<T, F>(&self, label: &'static str, f: F) -> Result<T, AcpError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        let conn = Arc::clone(&self.write_conn);
        tokio::task::spawn_blocking(move || {
            let c = conn
                .lock()
                .map_err(|_| AcpError::Internal(format!("write mutex poisoned ({label})")))?;
            f(&c).map_err(|e| {
                AcpError::Persistence(PersistenceError::Sqlite(format!("{label}: {e}")))
            })
        })
        .await
        .map_err(|e| AcpError::Internal(format!("{label} join: {e}")))?
    }

    async fn blocking_read<T, F>(&self, label: &'static str, f: F) -> Result<T, AcpError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        let conn = Arc::clone(&self.read_conn);
        tokio::task::spawn_blocking(move || {
            let c = conn
                .lock()
                .map_err(|_| AcpError::Internal(format!("read mutex poisoned ({label})")))?;
            f(&c).map_err(|e| {
                AcpError::Persistence(PersistenceError::Sqlite(format!("{label}: {e}")))
            })
        })
        .await
        .map_err(|e| AcpError::Internal(format!("{label} join: {e}")))?
    }
}

// ── AcpPersistence impl ───────────────────────────────────────────────────────

impl AcpPersistence for SqliteAcpPersistence {
    async fn load_sessions(&self) -> Result<Vec<AcpSessionSummary>, AcpError> {
        self.blocking_read("load_sessions", |c| db_load_sessions(c))
            .await
    }

    async fn load_events(&self, session_id: &str) -> Result<Vec<AcpEvent>, AcpError> {
        let sid = session_id.to_owned();
        self.blocking_read("load_events", move |c| db_load_events(c, &sid, None))
            .await
    }

    async fn load_events_since(
        &self,
        session_id: &str,
        since_seq: u64,
    ) -> Result<Vec<AcpEvent>, AcpError> {
        let sid = session_id.to_owned();
        self.blocking_read("load_events_since", move |c| {
            db_load_events(c, &sid, Some(since_seq))
        })
        .await
    }

    async fn save_session(&self, summary: &AcpSessionSummary) -> Result<(), AcpError> {
        let summary = summary.clone();
        self.blocking_write("save_session", move |c| db_save_session(c, &summary))
            .await
    }

    async fn append_event(&self, event: &AcpEvent) -> Result<(), AcpError> {
        // Sign PermissionOutcome events before storing.
        let event = maybe_sign_permission_outcome(event, &self.hmac_key);
        // Serialize once, redact the JSON tree in place; no from_value round-trip.
        let payload = redact_event_payload(&event)
            .map_err(|e| AcpError::Internal(format!("serialize event: {e}")))?;

        self.event_tx
            .send(PersistCmd::AppendEvent(event, payload))
            .await
            .map_err(|_| AcpError::Internal("event writer task channel closed".to_string()))
    }

    async fn update_session_state(
        &self,
        session_id: &str,
        state: AcpSessionState,
    ) -> Result<(), AcpError> {
        let sid = session_id.to_owned();
        self.blocking_write("update_session_state", move |c| {
            db_update_session_state(c, &sid, &state)
        })
        .await
    }
}

// ── Background writer task ────────────────────────────────────────────────────

async fn writer_task(conn: Arc<Mutex<Connection>>, mut rx: mpsc::Receiver<PersistCmd>) {
    let mut batch: Vec<(AcpEvent, String)> = Vec::with_capacity(BATCH_SIZE);

    loop {
        // Collect up to BATCH_SIZE events within BATCH_TIMEOUT.
        let deadline = tokio::time::Instant::now() + BATCH_TIMEOUT;

        loop {
            match tokio::time::timeout_at(deadline, rx.recv()).await {
                Ok(Some(PersistCmd::AppendEvent(ev, payload))) => {
                    batch.push((ev, payload));
                    if batch.len() >= BATCH_SIZE {
                        break;
                    }
                }
                Ok(Some(PersistCmd::Flush(tx))) => {
                    // Flush whatever we have now, then ack.
                    flush_batch(&conn, &mut batch).await;
                    tx.send(Ok(())).ok();
                    break;
                }
                Ok(None) => {
                    // Channel closed — flush and exit task.
                    flush_batch(&conn, &mut batch).await;
                    return;
                }
                Err(_) => {
                    // Timeout — flush partial batch.
                    break;
                }
            }
        }

        if !batch.is_empty() {
            flush_batch(&conn, &mut batch).await;
        }
    }
}

async fn flush_batch(conn: &Arc<Mutex<Connection>>, batch: &mut Vec<(AcpEvent, String)>) {
    if batch.is_empty() {
        return;
    }
    let events = std::mem::take(batch);
    let count = events.len();
    let conn = Arc::clone(conn);
    let result = tokio::task::spawn_blocking(move || {
        let c = conn.lock().map_err(|_| "writer mutex poisoned".to_string())?;
        db_batch_insert_events(&c, &events)
            .map_err(|e| format!("batch insert events: {e}"))
    })
    .await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::error!(
            surface = "acp",
            service = "persistence",
            action = "flush_batch",
            kind = "internal_error",
            events = count,
            error,
            "acp event batch insert failed",
        ),
        Err(join_err) => tracing::error!(
            surface = "acp",
            service = "persistence",
            action = "flush_batch",
            kind = "internal_error",
            events = count,
            error = %join_err,
            "acp event flush task panicked",
        ),
    }
}

// ── Database helpers ──────────────────────────────────────────────────────────

fn apply_wal_pragmas(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA mmap_size=134217728;
        PRAGMA cache_size=-65536;
        PRAGMA wal_autocheckpoint=1000;
        ",
    )
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(SCHEMA_SQL)?;
        conn.pragma_update(None, "user_version", 1)?;
    }
    Ok(())
}

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS acp_sessions (
    id                 TEXT PRIMARY KEY,
    provider           TEXT NOT NULL,
    title              TEXT NOT NULL,
    cwd                TEXT NOT NULL,
    state              TEXT NOT NULL,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    principal          TEXT NOT NULL DEFAULT '',
    agent_name         TEXT,
    agent_version      TEXT,
    provider_session_id TEXT
);

CREATE TABLE IF NOT EXISTS acp_session_events (
    id         TEXT PRIMARY KEY,
    session_id TEXT    NOT NULL REFERENCES acp_sessions(id),
    seq        INTEGER NOT NULL,
    kind       TEXT    NOT NULL,
    created_at TEXT    NOT NULL,
    payload    TEXT    NOT NULL,
    UNIQUE(session_id, seq)
);
CREATE INDEX IF NOT EXISTS idx_events_session_seq
    ON acp_session_events(session_id, seq);

CREATE TABLE IF NOT EXISTS acp_permission_requests (
    id             TEXT PRIMARY KEY,
    session_id     TEXT NOT NULL REFERENCES acp_sessions(id),
    action_summary TEXT NOT NULL,
    options        TEXT NOT NULL,
    outcome        TEXT,
    created_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_perm_session
    ON acp_permission_requests(session_id);
";

// ── Session CRUD ──────────────────────────────────────────────────────────────

fn db_load_sessions(conn: &Connection) -> rusqlite::Result<Vec<AcpSessionSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, title, cwd, state, created_at, updated_at,
                principal, agent_name, agent_version, provider_session_id
         FROM acp_sessions
         ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let state_str: String = row.get("state")?;
        let state = parse_session_state(&state_str);
        let principal: String = row.get("principal")?;
        Ok(AcpSessionSummary {
            id: row.get("id")?,
            provider: row.get("provider")?,
            title: row.get("title")?,
            cwd: row.get("cwd")?,
            state,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            principal: if principal.is_empty() {
                None
            } else {
                Some(principal)
            },
            agent_name: row.get("agent_name")?,
            agent_version: row.get("agent_version")?,
            provider_session_id: row.get("provider_session_id")?,
        })
    })?;
    rows.collect()
}

fn db_save_session(conn: &Connection, s: &AcpSessionSummary) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO acp_sessions
             (id, provider, title, cwd, state, created_at, updated_at,
              principal, agent_name, agent_version, provider_session_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
             provider           = excluded.provider,
             title              = excluded.title,
             cwd                = excluded.cwd,
             state              = excluded.state,
             updated_at         = excluded.updated_at,
             principal          = excluded.principal,
             agent_name         = excluded.agent_name,
             agent_version      = excluded.agent_version,
             provider_session_id = excluded.provider_session_id",
        params![
            s.id,
            s.provider,
            s.title,
            s.cwd,
            session_state_str(&s.state),
            s.created_at,
            s.updated_at,
            s.principal.as_deref().unwrap_or(""),
            s.agent_name,
            s.agent_version,
            s.provider_session_id,
        ],
    )?;
    Ok(())
}

fn db_update_session_state(
    conn: &Connection,
    session_id: &str,
    state: &AcpSessionState,
) -> rusqlite::Result<()> {
    let now = jiff::Timestamp::now().to_string();
    conn.execute(
        "UPDATE acp_sessions SET state = ?1, updated_at = ?2 WHERE id = ?3",
        params![session_state_str(state), now, session_id],
    )?;
    Ok(())
}

// ── Event CRUD ────────────────────────────────────────────────────────────────

fn db_load_events(
    conn: &Connection,
    session_id: &str,
    since_seq: Option<u64>,
) -> rusqlite::Result<Vec<AcpEvent>> {
    let (sql, args): (&str, Vec<rusqlite::types::Value>) = match since_seq {
        None => (
            "SELECT id, seq, kind, created_at, payload
             FROM acp_session_events
             WHERE session_id = ?1
             ORDER BY seq ASC",
            vec![session_id.to_owned().into()],
        ),
        Some(since) => (
            "SELECT id, seq, kind, created_at, payload
             FROM acp_session_events
             WHERE session_id = ?1 AND seq > ?2
             ORDER BY seq ASC",
            vec![session_id.to_owned().into(), (since as i64).into()],
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
        let payload: String = row.get("payload")?;
        Ok(payload)
    })?;

    let mut events = Vec::new();
    for row in rows {
        let payload = row?;
        if let Ok(ev) = serde_json::from_str::<AcpEvent>(&payload) {
            events.push(ev);
        }
        // Silently skip rows that fail to deserialise (schema evolution).
    }
    Ok(events)
}

fn db_batch_insert_events(
    conn: &Connection,
    events: &[(AcpEvent, String)],
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin transaction: {e}"))?;

    for (event, payload) in events {
        let id = event_id(event);
        let session_id = event.session_id();
        let seq = event.seq() as i64;
        let kind = event_kind_str(event);
        let created_at = event_created_at(event);

        tx.execute(
            "INSERT OR IGNORE INTO acp_session_events
                 (id, session_id, seq, kind, created_at, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, session_id, seq, kind, created_at, payload],
        )
        .map_err(|e| format!("insert event: {e}"))?;
    }

    tx.commit().map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

// ── Event field accessors ─────────────────────────────────────────────────────

fn event_id(event: &AcpEvent) -> &str {
    match event {
        AcpEvent::MessageChunk { id, .. }
        | AcpEvent::ReasoningChunk { id, .. }
        | AcpEvent::ToolCallStart { id, .. }
        | AcpEvent::ToolCallUpdate { id, .. }
        | AcpEvent::PermissionRequest { id, .. }
        | AcpEvent::PermissionOutcome { id, .. }
        | AcpEvent::UsageUpdate { id, .. }
        | AcpEvent::ContentBlocks { id, .. }
        | AcpEvent::SessionUpdate { id, .. }
        | AcpEvent::ProviderInfo { id, .. }
        | AcpEvent::Unknown { id, .. } => id,
    }
}

fn event_kind_str(event: &AcpEvent) -> &'static str {
    match event {
        AcpEvent::MessageChunk { .. } => "message_chunk",
        AcpEvent::ReasoningChunk { .. } => "reasoning_chunk",
        AcpEvent::ToolCallStart { .. } => "tool_call_start",
        AcpEvent::ToolCallUpdate { .. } => "tool_call_update",
        AcpEvent::PermissionRequest { .. } => "permission_request",
        AcpEvent::PermissionOutcome { .. } => "permission_outcome",
        AcpEvent::UsageUpdate { .. } => "usage_update",
        AcpEvent::ContentBlocks { .. } => "content_blocks",
        AcpEvent::SessionUpdate { .. } => "session_update",
        AcpEvent::ProviderInfo { .. } => "provider_info",
        AcpEvent::Unknown { .. } => "unknown",
    }
}

fn event_created_at(event: &AcpEvent) -> &str {
    match event {
        AcpEvent::MessageChunk { created_at, .. }
        | AcpEvent::ReasoningChunk { created_at, .. }
        | AcpEvent::ToolCallStart { created_at, .. }
        | AcpEvent::ToolCallUpdate { created_at, .. }
        | AcpEvent::PermissionRequest { created_at, .. }
        | AcpEvent::PermissionOutcome { created_at, .. }
        | AcpEvent::UsageUpdate { created_at, .. }
        | AcpEvent::ContentBlocks { created_at, .. }
        | AcpEvent::SessionUpdate { created_at, .. }
        | AcpEvent::ProviderInfo { created_at, .. }
        | AcpEvent::Unknown { created_at, .. } => created_at,
    }
}

// ── Session state helpers ─────────────────────────────────────────────────────

fn session_state_str(state: &AcpSessionState) -> &'static str {
    match state {
        AcpSessionState::Creating => "creating",
        AcpSessionState::Idle => "idle",
        AcpSessionState::Running => "running",
        AcpSessionState::WaitingForPermission => "waiting_for_permission",
        AcpSessionState::Completed => "completed",
        AcpSessionState::Cancelled => "cancelled",
        AcpSessionState::Failed => "failed",
        AcpSessionState::Closed => "closed",
    }
}

fn parse_session_state(s: &str) -> AcpSessionState {
    match s {
        "creating" => AcpSessionState::Creating,
        "idle" => AcpSessionState::Idle,
        "running" => AcpSessionState::Running,
        "waiting_for_permission" => AcpSessionState::WaitingForPermission,
        "completed" => AcpSessionState::Completed,
        "cancelled" => AcpSessionState::Cancelled,
        "failed" => AcpSessionState::Failed,
        "closed" => AcpSessionState::Closed,
        _ => AcpSessionState::Failed,
    }
}

// ── HMAC permission outcome signing ──────────────────────────────────────────

/// If the event is a `PermissionOutcome`, append an HMAC tag to the `raw`
/// field (in-memory only — the signed form is what gets persisted).
fn maybe_sign_permission_outcome(event: &AcpEvent, key: &[u8]) -> AcpEvent {
    if let AcpEvent::PermissionOutcome {
        id,
        created_at,
        session_id,
        seq,
        request_id,
        granted,
    } = event
    {
        let tag = hmac_tag(key, &format!("{id}:{request_id}:{granted}:{seq}"));
        // Reconstruct as Unknown so we can carry the HMAC tag without changing
        // the AcpEvent enum. The tag is embedded in `raw`.
        AcpEvent::PermissionOutcome {
            id: id.clone(),
            created_at: created_at.clone(),
            session_id: session_id.clone(),
            seq: *seq,
            request_id: format!("{request_id};hmac={tag}"),
            granted: *granted,
        }
    } else {
        event.clone()
    }
}

fn hmac_tag(key: &[u8], message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Load the HMAC secret from `LAB_ACP_HMAC_SECRET` or generate an ephemeral
/// key from process ID + startup timestamp hashed through SHA-256.
///
/// The generated key is NOT cryptographically random — it is only suitable
/// for protecting against naive DB-write bypass within a single process
/// lifetime. For cross-restart signature verification, set
/// `LAB_ACP_HMAC_SECRET` in `~/.lab/.env`.
fn load_or_generate_hmac_key() -> Vec<u8> {
    if let Ok(secret) = std::env::var("LAB_ACP_HMAC_SECRET") {
        if !secret.is_empty() {
            return secret.into_bytes();
        }
    }
    // Derive a 32-byte key from process ID + monotonic time via SHA-256.
    // This is NOT cryptographically random but is unique per process
    // instance. For production, set LAB_ACP_HMAC_SECRET.
    use sha2::Digest;
    let pid = std::process::id();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let input = format!("lab-acp-hmac-ephemeral:{pid}:{now}");
    Sha256::digest(input.as_bytes()).to_vec()
}

// ── Payload redaction ─────────────────────────────────────────────────────────

/// Serialize an event and redact sensitive fields in the JSON tree.
/// Returns the final payload string used for the DB `payload` column,
/// without a `from_value` round-trip back to the typed event.
fn redact_event_payload(event: &AcpEvent) -> serde_json::Result<String> {
    let mut value = serde_json::to_value(event)?;
    redact_value(&mut value);
    serde_json::to_string(&value)
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if crate::dispatch::redact::is_sensitive_key(key) {
                    *val = Value::String("[REDACTED]".to_string());
                } else {
                    redact_value(val);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_value(item);
            }
        }
        _ => {}
    }
}

// ── Path resolution ───────────────────────────────────────────────────────────

fn resolve_db_path() -> Result<PathBuf, AcpError> {
    if let Ok(path) = std::env::var("LAB_ACP_DB") {
        crate::dispatch::helpers::reject_path_traversal(&path)
            .map_err(|_| AcpError::Internal(format!("LAB_ACP_DB must not contain `..`: {path}")))?;
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Ok(PathBuf::from(home).join(".lab").join("acp.db"))
}

// ── Unix file creation with 0600 perms ────────────────────────────────────────

#[cfg(unix)]
fn create_db_file_0600(path: &PathBuf) {
    use std::os::unix::fs::OpenOptionsExt;
    // Only set mode on creation; if the file already exists, leave perms alone.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true) // fails if file exists — that's fine
        .mode(0o600)
        .open(path)
        .ok();
}
