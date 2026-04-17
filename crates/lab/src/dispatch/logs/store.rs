//! SQLite-backed persistence for captured log events.
//!
//! Single writer (the async writer task in `ingest`), many readers. WAL mode
//! + shared `Mutex<Connection>` inside `spawn_blocking` keeps the API async
//! without dragging in `sqlx`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, params, params_from_iter};

use super::types::{
    LogEvent, LogLevel, LogQuery, LogRetention, LogSearchResult, LogStoreStats, LogTailRequest,
    LogTailResult, Subsystem, Surface,
};
use crate::dispatch::error::ToolError;

const SCHEMA_SQL: &str = include_str!("store_schema.sql");

pub struct LogStore {
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) path: PathBuf,
    pub(super) retention: LogRetention,
}

impl LogStore {
    pub async fn open(path: PathBuf, retention: LogRetention) -> Result<Self, ToolError> {
        let conn_path = path.clone();
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection, rusqlite::Error> {
            if let Some(parent) = conn_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
            let conn = Connection::open_with_flags(&conn_path, flags)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "temp_store", "MEMORY")?;
            conn.pragma_update(None, "mmap_size", 134_217_728_i64)?;
            conn.execute_batch(SCHEMA_SQL)?;
            Ok(conn)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store open: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
            retention,
        })
    }

    pub async fn insert(&self, event: &LogEvent) -> Result<(), ToolError> {
        let conn = Arc::clone(&self.conn);
        let event = event.clone();
        tokio::task::spawn_blocking(move || -> Result<(), rusqlite::Error> {
            let c = conn.lock().expect("log store mutex poisoned");
            insert_event(&c, &event)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store insert join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store insert: {e}")))
    }

    pub async fn search(&self, query: LogQuery) -> Result<LogSearchResult, ToolError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<LogSearchResult, rusqlite::Error> {
            let c = conn.lock().expect("log store mutex poisoned");
            run_search(&c, &query)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store search join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store search: {e}")))
    }

    pub async fn tail(&self, req: LogTailRequest) -> Result<LogTailResult, ToolError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<LogTailResult, rusqlite::Error> {
            let c = conn.lock().expect("log store mutex poisoned");
            run_tail(&c, &req)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store tail join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store tail: {e}")))
    }

    pub async fn stats(&self) -> Result<LogStoreStats, ToolError> {
        let conn = Arc::clone(&self.conn);
        let retention = self.retention;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || -> Result<LogStoreStats, rusqlite::Error> {
            let c = conn.lock().expect("log store mutex poisoned");
            run_stats(&c, retention, &path)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store stats join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store stats: {e}")))
    }

    pub async fn run_maintenance(&self) -> Result<(), ToolError> {
        let conn = Arc::clone(&self.conn);
        let retention = self.retention;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), rusqlite::Error> {
            let c = conn.lock().expect("log store mutex poisoned");
            run_maintenance(&c, retention, &path)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("log store maintenance join: {e}")))?
        .map_err(|e| ToolError::internal_message(format!("log store maintenance: {e}")))
    }
}

#[doc(hidden)]
pub async fn open_store_for_test(retention: LogRetention) -> Result<LogStore, ToolError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "lab-logs-test-{}-{unique}.db",
        std::process::id()
    ));
    LogStore::open(path, retention).await
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

fn level_from_str(s: &str) -> Option<LogLevel> {
    Some(match s {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => return None,
    })
}

// ── Search ────────────────────────────────────────────────────────────────────

fn run_search(conn: &Connection, q: &LogQuery) -> Result<LogSearchResult, rusqlite::Error> {
    let mut sql = String::from(
        "SELECT event_id, ts, level, subsystem, surface, action, message,
                request_id, session_id, correlation_id, trace_id, span_id,
                instance, auth_flow, outcome_kind, fields_json,
                source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id
         FROM log_events WHERE 1=1",
    );
    let mut args: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(after) = q.after_ts {
        sql.push_str(" AND ts > ?");
        args.push(after.into());
    }
    if let Some(before) = q.before_ts {
        sql.push_str(" AND ts < ?");
        args.push(before.into());
    }
    if !q.levels.is_empty() {
        sql.push_str(" AND level IN (");
        for (i, lv) in q.levels.iter().enumerate() {
            if i > 0 {
                sql.push(',');
            }
            sql.push('?');
            args.push(lv.as_str().to_string().into());
        }
        sql.push(')');
    }
    if !q.subsystems.is_empty() {
        sql.push_str(" AND subsystem IN (");
        for (i, s) in q.subsystems.iter().enumerate() {
            if i > 0 {
                sql.push(',');
            }
            sql.push('?');
            args.push(s.as_str().to_string().into());
        }
        sql.push(')');
    }
    if !q.surfaces.is_empty() {
        sql.push_str(" AND surface IN (");
        for (i, s) in q.surfaces.iter().enumerate() {
            if i > 0 {
                sql.push(',');
            }
            sql.push('?');
            args.push(s.as_str().to_string().into());
        }
        sql.push(')');
    }
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
    Ok(LogSearchResult { events, next_cursor })
}

fn row_to_event(row: &rusqlite::Row<'_>) -> Result<LogEvent, rusqlite::Error> {
    let level: String = row.get("level")?;
    let subsystem: String = row.get("subsystem")?;
    let surface: String = row.get("surface")?;
    let fields_json: String = row.get("fields_json")?;
    Ok(LogEvent {
        event_id: row.get("event_id")?,
        ts: row.get("ts")?,
        level: level_from_str(&level).unwrap_or(LogLevel::Info),
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
    let limit = req.limit.unwrap_or(500).min(10_000);

    let (sql, args): (String, Vec<rusqlite::types::Value>) = match (cursor_ts, cursor_event_id) {
        (Some(ts), Some(ev_id)) => (
            "SELECT event_id, ts, level, subsystem, surface, action, message,
                    request_id, session_id, correlation_id, trace_id, span_id,
                    instance, auth_flow, outcome_kind, fields_json,
                    source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id
             FROM log_events
             WHERE ts > ?1 OR (ts = ?1 AND event_id > ?2)
             ORDER BY ts ASC, event_id ASC
             LIMIT ?3"
                .into(),
            vec![ts.into(), ev_id.into(), (limit as i64).into()],
        ),
        (Some(ts), None) => (
            "SELECT event_id, ts, level, subsystem, surface, action, message,
                    request_id, session_id, correlation_id, trace_id, span_id,
                    instance, auth_flow, outcome_kind, fields_json,
                    source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id
             FROM log_events
             WHERE ts > ?1
             ORDER BY ts ASC, event_id ASC
             LIMIT ?2"
                .into(),
            vec![ts.into(), (limit as i64).into()],
        ),
        (None, _) => (
            "SELECT event_id, ts, level, subsystem, surface, action, message,
                    request_id, session_id, correlation_id, trace_id, span_id,
                    instance, auth_flow, outcome_kind, fields_json,
                    source_kind, source_node_id, source_device_id, ingest_path, upstream_event_id
             FROM log_events
             ORDER BY ts ASC, event_id ASC
             LIMIT ?1"
                .into(),
            vec![(limit as i64).into()],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(args.iter()), row_to_event)?;
    let events: Vec<LogEvent> = rows.collect::<Result<_, _>>()?;
    let next_cursor = events.last().map(|e| e.event_id.clone());
    Ok(LogTailResult { events, next_cursor })
}

// ── Stats ─────────────────────────────────────────────────────────────────────

fn run_stats(
    conn: &Connection,
    retention: LogRetention,
    path: &Path,
) -> Result<LogStoreStats, rusqlite::Error> {
    let (total_event_count, oldest_retained_ts, newest_retained_ts): (u64, Option<i64>, Option<i64>) =
        conn.query_row(
            "SELECT COUNT(*), MIN(ts), MAX(ts) FROM log_events",
            [],
            |row| Ok((row.get::<_, u64>(0)?, row.get(1)?, row.get(2)?)),
        )?;

    let on_disk_bytes = sidecar_bytes(path);

    Ok(LogStoreStats {
        on_disk_bytes,
        oldest_retained_ts,
        newest_retained_ts,
        total_event_count,
        dropped_event_count: 0,
        retention,
    })
}

fn sidecar_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    for suffix in ["", "-wal", "-shm"] {
        let p = if suffix.is_empty() {
            path.to_path_buf()
        } else {
            let mut os = path.as_os_str().to_os_string();
            os.push(suffix);
            PathBuf::from(os)
        };
        if let Ok(meta) = std::fs::metadata(&p) {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

// ── Maintenance ───────────────────────────────────────────────────────────────

fn run_maintenance(
    conn: &Connection,
    retention: LogRetention,
    path: &Path,
) -> Result<(), rusqlite::Error> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(i64::MAX);
    let cutoff = now_ms.saturating_sub((retention.max_age_days as i64) * 86_400_000);
    conn.execute("DELETE FROM log_events WHERE ts < ?1", params![cutoff])?;

    for _ in 0..16 {
        if sidecar_bytes(path) <= retention.max_bytes {
            break;
        }
        let affected = conn.execute(
            "DELETE FROM log_events WHERE rowid IN
               (SELECT rowid FROM log_events ORDER BY ts ASC LIMIT 1024)",
            [],
        )?;
        if affected == 0 {
            break;
        }
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")?;
    }
    Ok(())
}
