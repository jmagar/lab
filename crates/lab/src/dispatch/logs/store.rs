//! SQLite-backed persistence for captured log events.
//!
//! Single writer (the async writer task in `ingest`), many readers. WAL mode
//! + shared `Mutex<Connection>` inside `spawn_blocking` keeps the API async
//! without dragging in `sqlx`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, params, params_from_iter};

use super::types::{
    LogEvent, LogLevel, LogQuery, LogRetention, LogSearchResult, LogStoreStats, LogTailRequest,
    LogTailResult, Subsystem, Surface,
};
use crate::dispatch::error::ToolError;

/// Column list shared by search + tail. Kept in sync with
/// `row_to_event`'s `row.get(...)` names.
const SELECT_COLS: &str = "event_id, ts, level, subsystem, surface, action, message,
     request_id, session_id, correlation_id, trace_id, span_id,
     instance, auth_flow, outcome_kind, fields_json,
     source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id";

pub struct LogStore {
    conn: Arc<Mutex<Connection>>,
    retention: LogRetention,
}

impl LogStore {
    pub async fn open(path: PathBuf, retention: LogRetention) -> Result<Self, ToolError> {
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection, rusqlite::Error> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
            let conn = Connection::open_with_flags(&path, flags)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "temp_store", "MEMORY")?;
            conn.pragma_update(None, "mmap_size", 134_217_728_i64)?;
            migrate(&conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store open join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store open: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            retention,
        })
    }

    pub async fn insert(&self, event: &LogEvent) -> Result<(), ToolError> {
        let event = event.clone();
        self.blocking("insert", move |c| insert_event(c, &event))
            .await
    }

    pub async fn search(&self, query: LogQuery) -> Result<LogSearchResult, ToolError> {
        self.blocking("search", move |c| run_search(c, &query))
            .await
    }

    pub async fn tail(&self, req: LogTailRequest) -> Result<LogTailResult, ToolError> {
        self.blocking("tail", move |c| run_tail(c, &req)).await
    }

    pub async fn stats(&self) -> Result<LogStoreStats, ToolError> {
        let retention = self.retention;
        self.blocking("stats", move |c| run_stats(c, retention))
            .await
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn run_maintenance(&self) -> Result<(), ToolError> {
        let retention = self.retention;
        self.blocking("maintenance", move |c| run_maintenance(c, retention))
            .await
    }

    /// Run a `Connection`-using closure on the blocking pool. Wraps the
    /// repeated `spawn_blocking → lock → double map_err` pattern.
    async fn blocking<T, F>(&self, label: &'static str, f: F) -> Result<T, ToolError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let c = conn.lock().expect("log store mutex poisoned");
            f(&c)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store {label} join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store {label}: {e}")))
    }
}

#[doc(hidden)]
#[allow(dead_code)]
pub async fn open_store_for_test(retention: LogRetention) -> Result<LogStore, ToolError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path =
        std::env::temp_dir().join(format!("lab-logs-test-{}-{unique}.db", std::process::id()));
    LogStore::open(path, retention).await
}

// ── Schema migration ──────────────────────────────────────────────────────────

/// Apply pending schema migrations using PRAGMA user_version as the version
/// counter. Each `if version < N` block is a single, idempotent migration step.
///
/// Rules:
/// - Only bump `user_version` **after** the DDL succeeds.
/// - Keep version numbers consecutive and never reuse them.
/// - Future columns: `if version < 2 { conn.execute_batch("ALTER TABLE ...")?; ... }`
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(include_str!("store_schema.sql"))?;
        conn.pragma_update(None, "user_version", 1)?;
    }
    // Future migrations:
    // if version < 2 {
    //     conn.execute_batch("ALTER TABLE log_events ADD COLUMN new_col TEXT;")?;
    //     conn.pragma_update(None, "user_version", 2)?;
    // }
    Ok(())
}

// ── Insert ────────────────────────────────────────────────────────────────────

fn insert_event(conn: &Connection, event: &LogEvent) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO log_events (
            event_id, ts, level, subsystem, surface, action, message,
            request_id, session_id, correlation_id, trace_id, span_id,
            instance, auth_flow, outcome_kind, fields_json,
            source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
        params![
            event.event_id,
            event.ts,
            event.level.as_str(),
            event.subsystem.as_str(),
            event.surface.as_str(),
            event.action,
            event.message,
            event.request_id,
            event.session_id,
            event.correlation_id,
            event.trace_id,
            event.span_id,
            event.instance,
            event.auth_flow,
            event.outcome_kind,
            event.fields_json.to_string(),
            event.source_kind,
            event.source_node_id,
            event.source_device_id,
            event.ingest_path,
            event.upstream_event_id,
        ],
    )?;
    Ok(())
}

// ── Search ────────────────────────────────────────────────────────────────────

fn run_search(conn: &Connection, q: &LogQuery) -> Result<LogSearchResult, rusqlite::Error> {
    let mut sql = format!("SELECT {SELECT_COLS} FROM log_events WHERE 1=1");
    let mut args: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(after) = q.after_ts {
        sql.push_str(" AND ts > ?");
        args.push(after.into());
    }
    if let Some(before) = q.before_ts {
        sql.push_str(" AND ts < ?");
        args.push(before.into());
    }
    append_in_clause(
        &mut sql,
        &mut args,
        "level",
        q.levels.iter().map(|l| l.as_str().to_string()),
    );
    append_in_clause(
        &mut sql,
        &mut args,
        "subsystem",
        q.subsystems.iter().map(|s| s.as_str().to_string()),
    );
    append_in_clause(
        &mut sql,
        &mut args,
        "surface",
        q.surfaces.iter().map(|s| s.as_str().to_string()),
    );
    if let Some(action) = &q.action {
        sql.push_str(" AND action = ?");
        args.push(action.clone().into());
    }
    if let Some(request_id) = &q.request_id {
        sql.push_str(" AND request_id = ?");
        args.push(request_id.clone().into());
    }
    if let Some(session_id) = &q.session_id {
        sql.push_str(" AND session_id = ?");
        args.push(session_id.clone().into());
    }
    if let Some(corr) = &q.correlation_id {
        sql.push_str(" AND correlation_id = ?");
        args.push(corr.clone().into());
    }
    append_in_clause(
        &mut sql,
        &mut args,
        "source_node_id",
        q.source_node_ids.iter().cloned(),
    );
    append_in_clause(
        &mut sql,
        &mut args,
        "source_kind",
        q.source_kinds.iter().cloned(),
    );
    if let Some(text) = &q.text {
        sql.push_str(
            " AND (message LIKE ? OR IFNULL(request_id,'') LIKE ? OR IFNULL(session_id,'') LIKE ? OR IFNULL(correlation_id,'') LIKE ?)",
        );
        let like = format!("%{text}%");
        args.push(like.clone().into());
        args.push(like.clone().into());
        args.push(like.clone().into());
        args.push(like.into());
    }
    sql.push_str(" ORDER BY ts DESC, event_id DESC");
    let limit = q.limit.unwrap_or(500).min(10_000);
    sql.push_str(&format!(" LIMIT {limit}"));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), row_to_event)?;
    let events: Vec<LogEvent> = rows.collect::<Result<_, _>>()?;
    let next_cursor = events.last().map(|e| e.event_id.clone());
    Ok(LogSearchResult {
        events,
        next_cursor,
    })
}

fn append_in_clause<I>(
    sql: &mut String,
    args: &mut Vec<rusqlite::types::Value>,
    column: &str,
    values: I,
) where
    I: IntoIterator<Item = String>,
{
    let values: Vec<String> = values.into_iter().collect();
    if values.is_empty() {
        return;
    }
    sql.push_str(&format!(" AND {column} IN ("));
    for (i, v) in values.into_iter().enumerate() {
        if i > 0 {
            sql.push(',');
        }
        sql.push('?');
        args.push(v.into());
    }
    sql.push(')');
}

fn row_to_event(row: &rusqlite::Row<'_>) -> Result<LogEvent, rusqlite::Error> {
    let level: String = row.get("level")?;
    let subsystem: String = row.get("subsystem")?;
    let surface: String = row.get("surface")?;
    let fields_json: String = row.get("fields_json")?;
    Ok(LogEvent {
        event_id: row.get("event_id")?,
        ts: row.get("ts")?,
        level: LogLevel::parse(&level).unwrap_or(LogLevel::Info),
        subsystem: Subsystem::parse(&subsystem).unwrap_or(Subsystem::CoreRuntime),
        surface: Surface::parse(&surface).unwrap_or(Surface::CoreRuntime),
        action: row.get("action")?,
        message: row.get("message")?,
        request_id: row.get("request_id")?,
        session_id: row.get("session_id")?,
        correlation_id: row.get("correlation_id")?,
        trace_id: row.get("trace_id")?,
        span_id: row.get("span_id")?,
        instance: row.get("instance")?,
        auth_flow: row.get("auth_flow")?,
        outcome_kind: row.get("outcome_kind")?,
        fields_json: serde_json::from_str(&fields_json).unwrap_or(serde_json::Value::Null),
        source_kind: row.get("source_kind")?,
        source_node_id: row.get("source_node_id")?,
        source_device_id: row.get("source_device_id")?,
        ingest_path: row.get("ingest_path")?,
        upstream_event_id: row.get("upstream_event_id")?,
    })
}

// ── Tail ──────────────────────────────────────────────────────────────────────

fn run_tail(conn: &Connection, req: &LogTailRequest) -> Result<LogTailResult, rusqlite::Error> {
    let mut cursor_ts: Option<i64> = req.after_ts;
    let mut cursor_event_id: Option<String> = None;
    if let Some(ev_id) = &req.since_event_id {
        let ts: Option<i64> = conn
            .query_row(
                "SELECT ts FROM log_events WHERE event_id = ?1",
                params![ev_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(t) = ts {
            cursor_ts = Some(t);
            cursor_event_id = Some(ev_id.clone());
        }
    }
    let limit = req.limit.unwrap_or(500).min(10_000) as i64;

    // NOTE: the `ts > ?1 OR (ts = ?1 AND event_id > ?2)` tiebreaker is what
    // makes the `since_event_id` cursor ordering stable. Do not collapse.
    let (where_clause, args): (&str, Vec<rusqlite::types::Value>) =
        match (cursor_ts, cursor_event_id) {
            (Some(ts), Some(ev_id)) => (
                "WHERE ts > ?1 OR (ts = ?1 AND event_id > ?2)",
                vec![ts.into(), ev_id.into(), limit.into()],
            ),
            (Some(ts), None) => ("WHERE ts > ?1", vec![ts.into(), limit.into()]),
            (None, _) => ("", vec![limit.into()]),
        };

    let limit_placeholder = args.len();
    let sql = format!(
        "SELECT {SELECT_COLS} FROM log_events {where_clause} \
         ORDER BY ts ASC, event_id ASC LIMIT ?{limit_placeholder}"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), row_to_event)?;
    let events: Vec<LogEvent> = rows.collect::<Result<_, _>>()?;
    let next_cursor = events.last().map(|e| e.event_id.clone());
    Ok(LogTailResult {
        events,
        next_cursor,
    })
}

// ── Stats ─────────────────────────────────────────────────────────────────────

fn run_stats(conn: &Connection, retention: LogRetention) -> Result<LogStoreStats, rusqlite::Error> {
    let (total_event_count, oldest_retained_ts, newest_retained_ts): (
        u64,
        Option<i64>,
        Option<i64>,
    ) = conn.query_row(
        "SELECT COUNT(*), MIN(ts), MAX(ts) FROM log_events",
        [],
        |row| Ok((row.get::<_, u64>(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let on_disk_bytes = content_bytes(conn)?;

    Ok(LogStoreStats {
        on_disk_bytes,
        oldest_retained_ts,
        newest_retained_ts,
        total_event_count,
        dropped_event_count: 0,
        retention,
    })
}

/// Sum of the logical content bytes retained (message + fields_json + small
/// per-row overhead). This is what retention policies act on — NOT the
/// physical SQLite file size, which has fixed overhead + WAL sidecar weight
/// that can't be shrunk below a few KB regardless of content.
fn content_bytes(conn: &Connection) -> Result<u64, rusqlite::Error> {
    conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(message) + LENGTH(fields_json)), 0) FROM log_events",
        [],
        |row| row.get::<_, i64>(0).map(|n| n.max(0) as u64),
    )
}

// ── Maintenance ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn run_maintenance(conn: &Connection, retention: LogRetention) -> Result<(), rusqlite::Error> {
    let now_ms = super::ingest::now_ms();
    let age_ms = i64::try_from(retention.max_age_days)
        .ok()
        .and_then(|d| d.checked_mul(86_400_000))
        .unwrap_or(i64::MAX);
    let cutoff = now_ms.saturating_sub(age_ms);
    conn.execute("DELETE FROM log_events WHERE ts < ?1", params![cutoff])?;

    for _ in 0..64 {
        if content_bytes(conn)? <= retention.max_bytes {
            break;
        }
        let affected = conn.execute(
            "DELETE FROM log_events WHERE rowid IN
               (SELECT rowid FROM log_events ORDER BY ts ASC LIMIT 256)",
            [],
        )?;
        if affected == 0 {
            break;
        }
    }
    // Checkpoint the WAL to reclaim pages and keep the WAL file small.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    Ok(())
}
