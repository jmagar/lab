//! SQLite-backed persistence for the MCP server registry.
//!
//! Uses an r2d2 connection pool with WAL mode for concurrent access from
//! multiple axum handlers. Pool size is 4 — sufficient for homelab workloads.
//! All pool operations run inside `tokio::task::spawn_blocking` since
//! `Pool::get()` is blocking.

use std::path::Path;
use std::time::Duration;

use base64::Engine as _;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use lab_apis::mcpregistry::McpRegistryClient;
use lab_apis::mcpregistry::types::{ListServersParams, ServerJSON, ServerResponse};

use super::params::{SortBy, SortSpec};

/// Errors produced by [`RegistryStore`] and its helpers.
#[derive(Debug, Error)]
pub enum RegistryStoreError {
    #[error("sqlite error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("blocking task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    #[error("upstream registry error: {0}")]
    Upstream(String),
}

/// Internal cursor representation — base64-encoded JSON in wire form.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct CursorData {
    /// `server_name` of the last row on the previous page.
    s: String,
    /// `version` of the last row on the previous page.
    v: String,
}

/// DB-side list params — distinct from the upstream API `ListServersParams`.
///
/// Search input is capped at 512 bytes here to prevent DoS via enormous LIKE
/// pattern expansion.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct StoreListParams {
    /// Substring search on server_name (max 512 bytes enforced at construction).
    pub search: Option<String>,
    /// Opaque pagination cursor from a previous `PagedServers.next_cursor`.
    pub cursor: Option<String>,
    /// Page size — clamped server-side to `[1, 100]`; default 20.
    pub limit: Option<u32>,
    /// Sort specification.
    pub sort: Option<SortSpec>,
    /// Include rows with `status = 'deleted'`. Default: false.
    pub include_deleted: bool,
}

impl StoreListParams {
    /// Enforce the 512-byte search input cap, truncating silently.
    ///
    /// LEARNED: truncating at 512 bytes rather than rejecting prevents user-visible
    /// errors from over-long pastes while still bounding LIKE pattern expansion cost.
    #[must_use]
    pub fn with_search(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        self.search = if s.len() > 512 { Some(s[..512].to_string()) } else { Some(s) };
        self
    }
}

/// Paginated result from `RegistryStore::list_servers`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PagedServers {
    pub servers: Vec<ServerJSON>,
    /// Opaque cursor for the next page; `None` when this is the last page.
    pub next_cursor: Option<String>,
}

/// SQLite-backed store for registry server records.
///
/// Cloning the store is cheap — the inner `Pool` is `Arc`-backed.
///
/// Query methods are used by bead lab-h5pm.3+ consumers — `dead_code` is
/// expected until those consumers are wired in.
#[allow(dead_code)]
pub struct RegistryStore {
    pool: Pool<SqliteConnectionManager>,
}

impl Clone for RegistryStore {
    fn clone(&self) -> Self {
        Self { pool: self.pool.clone() }
    }
}

#[allow(dead_code)]
impl RegistryStore {
    /// Open (or create) the registry database at `path`.
    ///
    /// Sets file permissions to 0o600, enables WAL mode, and runs schema
    /// migrations inside a `BEGIN EXCLUSIVE` transaction to prevent TOCTOU
    /// races when two processes start concurrently.
    pub async fn open(path: &Path) -> Result<Self, RegistryStoreError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            // Ensure parent directory exists.
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Per-connection init: set busy_timeout, foreign_keys, synchronous.
            // These are connection-local pragmas and must be set on every
            // connection that the pool hands out.
            let manager =
                SqliteConnectionManager::file(&path).with_init(|conn| {
                    conn.execute_batch(
                        "PRAGMA busy_timeout = 5000;\
                         PRAGMA foreign_keys = ON;\
                         PRAGMA synchronous = NORMAL;",
                    )
                });

            let pool = Pool::builder()
                .max_size(4)
                .connection_timeout(Duration::from_secs(5))
                .build(manager)?;

            // Lock the file to 0o600 before writing any data.
            set_restrictive_permissions(&path)?;

            // Run schema migration on a freshly acquired connection.
            let conn = pool.get()?;
            migrate(&conn)?;
            drop(conn);

            Ok(Self { pool })
        })
        .await?
    }

    /// Acquire a pooled connection for use inside a `spawn_blocking` closure.
    ///
    /// This is intentionally not public — callers should use higher-level
    /// methods. It is `pub(crate)` so the dispatch module can reach it without
    /// exposing `r2d2` types to the rest of lab.
    #[allow(dead_code)]
    pub(crate) fn pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }

    // ── Query methods ─────────────────────────────────────────────────────────

    /// List servers with optional cursor pagination, LIKE search, status filter,
    /// and sort.
    ///
    /// Returns at most `params.limit` (clamped to `[1, 100]`) rows plus an
    /// opaque `next_cursor` when more rows exist.
    pub async fn list_servers(
        &self,
        params: StoreListParams,
    ) -> Result<PagedServers, RegistryStoreError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            list_servers_sync(&conn, &params)
        })
        .await?
    }

    /// Get a single server by name and version.
    ///
    /// Pass `version = "latest"` to resolve via the `is_latest = 1` flag.
    /// Returns `None` when no matching row exists.
    pub async fn get_server(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<ServerJSON>, RegistryStoreError> {
        let pool = self.pool.clone();
        let name = name.to_string();
        let version = version.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            get_server_sync(&conn, &name, &version)
        })
        .await?
    }

    /// List all stored versions for a given server name, ordered by `version`
    /// (lexicographic — callers should not assume semver order).
    ///
    /// LEARNED: `ORDER BY version` is lexicographic on TEXT; "0.9.10" < "0.9.9"
    /// in byte order. Downstream consumers that need semver ordering must sort
    /// after retrieval.
    pub async fn list_versions(
        &self,
        name: &str,
    ) -> Result<Vec<ServerJSON>, RegistryStoreError> {
        let pool = self.pool.clone();
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            list_versions_sync(&conn, &name)
        })
        .await?
    }

    /// Upsert a batch of `ServerResponse` entries in a single transaction.
    ///
    /// `server_json` is re-serialized from the parsed struct — raw upstream
    /// bytes are never stored directly.
    ///
    /// Returns the count of rows upserted (equal to `servers.len()` since
    /// `INSERT OR REPLACE` always writes exactly one row per entry).
    ///
    /// LEARNED: `INSERT OR REPLACE` returns 2 (delete + insert) when a row is
    /// replaced, so using `execute()` return values would double-count replacements.
    /// Returning `servers.len()` directly is the correct count.
    pub async fn upsert_page(
        &self,
        servers: &[ServerResponse],
    ) -> Result<usize, RegistryStoreError> {
        if servers.is_empty() {
            return Ok(0);
        }
        let pool = self.pool.clone();
        let servers = servers.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let count = upsert_page_sync(&mut conn, &servers)?;
            Ok(count)
        })
        .await?
    }

    /// Recompute `is_latest` for all servers in a single batch UPDATE.
    ///
    /// Called once at the end of a full sync — not per-page — to avoid N+1 pattern.
    ///
    /// IMPORTANT: Uses the upstream `is_latest` flag that was stored during upsert.
    /// Does NOT recompute from `MAX(version)` — lexicographic MAX is incorrect for
    /// semver (e.g. "0.9.10" < "0.9.9" in byte order). Trusting upstream avoids
    /// this class of bug entirely.
    pub async fn update_is_latest(&self) -> Result<(), RegistryStoreError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            update_is_latest_sync(&mut conn)
        })
        .await?
    }

    /// Full resync from the upstream MCP registry.
    ///
    /// Cursor-paginates through all pages at 100 servers/page. For each page,
    /// upserts into the store (pool.get() happens inside spawn_blocking, NEVER
    /// at an HTTP await point). After all pages, runs a single batch
    /// `update_is_latest` pass.
    ///
    /// Returns total count of rows upserted across all pages.
    pub async fn sync_from_upstream(
        &self,
        client: &McpRegistryClient,
    ) -> Result<usize, RegistryStoreError> {
        let started_at = std::time::Instant::now();
        let mut total = 0usize;
        let mut page_num = 0usize;
        let mut cursor: Option<String> = None;

        tracing::info!(
            service = "mcpregistry",
            event = "sync.start",
            "starting full registry sync from upstream"
        );

        loop {
            let params = ListServersParams {
                limit: Some(100),
                cursor: cursor.clone(),
                ..Default::default()
            };

            // HTTP fetch — no pool connection held during this await.
            let page = client
                .list_servers(params)
                .await
                .map_err(|e| RegistryStoreError::Upstream(e.to_string()))?;

            let page_len = page.servers.len();
            page_num += 1;

            if page_len > 0 {
                total += self.upsert_page(&page.servers).await?;
                tracing::debug!(
                    service = "mcpregistry",
                    event = "sync.page",
                    page = page_num,
                    page_size = page_len,
                    total_so_far = total,
                    "upserted page"
                );
            }

            match page.metadata.next_cursor {
                Some(c) if !c.is_empty() => cursor = Some(c),
                _ => break,
            }

            // Safety valve: if upstream returns empty page but a cursor, stop.
            if page_len == 0 {
                break;
            }
        }

        // Single batch recompute of is_latest after all pages are stored.
        self.update_is_latest().await?;

        tracing::info!(
            service = "mcpregistry",
            event = "sync.finish",
            total_servers = total,
            pages = page_num,
            elapsed_ms = started_at.elapsed().as_millis(),
            "registry sync complete"
        );

        Ok(total)
    }
}

// ── Sync helpers (run inside spawn_blocking) ──────────────────────────────────

#[allow(dead_code)]
fn list_servers_sync(
    conn: &Connection,
    params: &StoreListParams,
) -> Result<PagedServers, RegistryStoreError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1) as usize;

    let mut sql =
        "SELECT server_name, version, server_json FROM registry_servers WHERE 1=1".to_string();
    let mut args: Vec<rusqlite::types::Value> = Vec::new();

    // Status filter.
    if !params.include_deleted {
        sql.push_str(" AND status != 'deleted'");
    }

    // LIKE search on server_name.
    if let Some(search) = &params.search {
        let escaped = escape_like(search);
        sql.push_str(" AND server_name LIKE '%' || ?  || '%' ESCAPE '\\'");
        args.push(escaped.into());
    }

    // Cursor: base64-decode → {"s": name, "v": version} → compound WHERE clause.
    if let Some(cursor_str) = &params.cursor {
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor_str.as_bytes())
            .map_err(|e| RegistryStoreError::InvalidCursor(format!("base64 decode: {e}")))?;
        let cursor_data: CursorData = serde_json::from_slice(&decoded)
            .map_err(|e| RegistryStoreError::InvalidCursor(format!("json decode: {e}")))?;
        // Compound tiebreaker: OR is load-bearing to handle equal server_name with different version.
        sql.push_str(
            " AND (server_name > ? OR (server_name = ? AND version > ?))",
        );
        args.push(cursor_data.s.clone().into());
        args.push(cursor_data.s.into());
        args.push(cursor_data.v.into());
    }

    // ORDER BY — hardcoded from enum match, never interpolated from user input.
    let order_col = match params.sort.as_ref().map(|s| &s.by) {
        Some(SortBy::Updated) => "upstream_updated_at",
        Some(SortBy::Published) => "upstream_updated_at", // published_at not in schema; fall back
        Some(SortBy::Name) | None => "server_name",
    };
    let order_dir = match params.sort.as_ref().map(|s| s.desc) {
        Some(true) => "DESC",
        _ => "ASC",
    };
    sql.push_str(&format!(" ORDER BY {order_col} {order_dir}, version ASC"));

    // Fetch limit+1 to detect whether a next page exists, then truncate.
    sql.push_str(&format!(" LIMIT {}", limit + 1));

    let mut stmt = conn.prepare(&sql)?;
    use rusqlite::params_from_iter;
    let mut rows: Vec<ServerJSON> = stmt
        .query_map(params_from_iter(args.iter()), |row| {
            let json_str: String = row.get(2)?;
            Ok(json_str)
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| serde_json::from_str::<ServerJSON>(&s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }))
        .collect::<Result<Vec<_>, _>>()?;

    // Determine next cursor before truncating.
    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }

    let next_cursor = if has_more {
        rows.last().map(|s| {
            let data = CursorData { s: s.name.clone(), v: s.version.clone() };
            let json = serde_json::to_vec(&data).unwrap_or_default();
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json)
        })
    } else {
        None
    };

    Ok(PagedServers { servers: rows, next_cursor })
}

#[allow(dead_code)]
fn get_server_sync(
    conn: &Connection,
    name: &str,
    version: &str,
) -> Result<Option<ServerJSON>, RegistryStoreError> {
    let result = if version == "latest" {
        conn.query_row(
            "SELECT server_json FROM registry_servers \
             WHERE server_name = ?1 AND is_latest = 1 LIMIT 1",
            rusqlite::params![name],
            |row| row.get::<_, String>(0),
        )
    } else {
        conn.query_row(
            "SELECT server_json FROM registry_servers \
             WHERE server_name = ?1 AND version = ?2 LIMIT 1",
            rusqlite::params![name, version],
            |row| row.get::<_, String>(0),
        )
    };

    match result {
        Ok(json_str) => {
            let server: ServerJSON = serde_json::from_str(&json_str)?;
            Ok(Some(server))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(RegistryStoreError::Db(e)),
    }
}

#[allow(dead_code)]
fn list_versions_sync(
    conn: &Connection,
    name: &str,
) -> Result<Vec<ServerJSON>, RegistryStoreError> {
    let mut stmt = conn.prepare(
        "SELECT server_json FROM registry_servers \
         WHERE server_name = ?1 ORDER BY version ASC",
    )?;

    let rows = stmt
        .query_map(rusqlite::params![name], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| serde_json::from_str::<ServerJSON>(&s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Upsert a batch of `ServerResponse` entries in a single transaction.
///
/// Re-serializes `resp.server` from the parsed struct — never stores raw upstream bytes.
#[allow(dead_code)]
fn upsert_page_sync(
    conn: &mut Connection,
    servers: &[ServerResponse],
) -> rusqlite::Result<usize> {
    let tx = conn.transaction()?;

    for resp in servers {
        let server_json = serde_json::to_string(&resp.server)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let is_latest = resp
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .map(|ext| ext.is_latest)
            .unwrap_or(false) as i32;

        let status = resp
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .map(|ext| ext.status.as_str())
            .unwrap_or("active")
            .to_string();

        let updated_at: Option<String> = resp
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .and_then(|ext| ext.updated_at.clone());

        let synced_at = chrono::Utc::now().to_rfc3339();

        tx.execute(
            "INSERT OR REPLACE INTO registry_servers
             (server_name, version, is_latest, status, server_json, upstream_updated_at, synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                resp.server.name,
                resp.server.version,
                is_latest,
                status,
                server_json,
                updated_at,
                synced_at,
            ],
        )?;
    }

    tx.commit()?;

    // LEARNED: INSERT OR REPLACE returns 2 on conflict (delete + insert), so
    // summing execute() return values would double-count replacements. Return
    // servers.len() directly — it is the correct count of logical rows written.
    Ok(servers.len())
}

/// Recompute `is_latest` in a single batch UPDATE.
///
/// Uses the upstream `is_latest` flag stored during upsert as the source of truth.
/// If upstream sent conflicting flags (two rows both is_latest=1), the one with the
/// lexicographically greatest version wins as a tiebreaker — acceptable because
/// upstream should guarantee at most one is_latest per server_name.
///
/// IMPORTANT: Does NOT use MAX(version) for recomputation — lexicographic MAX is
/// incorrect for semver ("0.9.10" < "0.9.9"). Trusting upstream avoids this bug.
#[allow(dead_code)]
fn update_is_latest_sync(conn: &mut Connection) -> Result<(), RegistryStoreError> {
    let tx = conn.transaction()?;

    // First, clear all is_latest flags.
    tx.execute_batch("UPDATE registry_servers SET is_latest = 0;")?;

    // Then, for each server_name, find the row(s) the upstream marked is_latest=1
    // and pick the one with the greatest version as tiebreaker.
    tx.execute_batch(
        "UPDATE registry_servers
         SET is_latest = 1
         WHERE ROWID IN (
             SELECT r.ROWID
             FROM registry_servers r
             WHERE r.is_latest = 1
               AND r.status != 'deleted'
               AND r.version = (
                   SELECT MAX(r2.version)
                   FROM registry_servers r2
                   WHERE r2.server_name = r.server_name
                     AND r2.is_latest = 1
                     AND r2.status != 'deleted'
               )
         );",
    )?;

    tx.commit()?;
    Ok(())
}

// ── LIKE escaping ─────────────────────────────────────────────────────────────

/// Escape `\`, `%`, and `_` in a user-supplied string so they are treated as
/// literals by a SQLite `LIKE` expression using `ESCAPE '\'`.
///
/// Escape order is critical: backslash must be escaped first to prevent
/// double-escaping the escape character itself.
fn escape_like(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

// ── Migration ─────────────────────────────────────────────────────────────────

/// Apply pending schema migrations using PRAGMA user_version as the version
/// counter.
///
/// **MUST use `BEGIN EXCLUSIVE`** to prevent a TOCTOU race: two processes
/// starting simultaneously can both read `user_version = 0` and both attempt
/// to run the schema DDL.  The exclusive lock serialises them so only one
/// applies the migration.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("BEGIN EXCLUSIVE;")?;
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(include_str!("store_schema.sql"))?;
        conn.pragma_update(None, "user_version", 1)?;
    }
    // Future migrations:
    // if version < 2 {
    //     conn.execute_batch("ALTER TABLE registry_servers ADD COLUMN new_col TEXT;")?;
    //     conn.pragma_update(None, "user_version", 2)?;
    // }
    conn.execute_batch("COMMIT;")?;
    Ok(())
}

// ── Permissions ───────────────────────────────────────────────────────────────

/// Set the database file to owner-read/write only (0o600).
///
/// Called immediately after pool creation — before any data is written — so
/// the file is never world-readable even momentarily.
fn set_restrictive_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    // On non-Unix targets permissions are a no-op (homelab is Linux-only).
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn temp_store() -> RegistryStore {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir()
            .join(format!("lab-registry-test-{}-{unique}.db", std::process::id()));
        RegistryStore::open(&path).await.unwrap()
    }

    #[tokio::test]
    async fn store_opens_and_migrates() {
        // Verify open() succeeds and migration runs without panic.
        let store = temp_store().await;
        // Pool is accessible.
        let conn = store.pool().get().unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1, "schema version must be 1 after migration");
    }

    #[tokio::test]
    async fn migration_is_idempotent() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir()
            .join(format!("lab-registry-idem-{}-{unique}.db", std::process::id()));
        // Open twice — second open re-runs migrate() which must be a no-op.
        let _ = RegistryStore::open(&path).await.unwrap();
        let store2 = RegistryStore::open(&path).await.unwrap();
        let conn = store2.pool().get().unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn store_sets_restrictive_permissions() {
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir()
            .join(format!("lab-registry-perm-{}-{unique}.db", std::process::id()));
        let _ = RegistryStore::open(&path).await.unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "DB file must be 0o600");
    }

    fn make_server_response(name: &str, version: &str, is_latest: bool) -> ServerResponse {
        use lab_apis::mcpregistry::types::{RegistryExtensions, ResponseMeta};
        ServerResponse {
            server: ServerJSON {
                schema: None,
                name: name.to_string(),
                title: None,
                description: format!("Test server {name}"),
                version: version.to_string(),
                packages: vec![],
                remotes: vec![],
                repository: None,
                icons: vec![],
                website_url: None,
            },
            meta: Some(ResponseMeta {
                official: Some(RegistryExtensions {
                    is_latest,
                    published_at: "2025-01-01T00:00:00Z".to_string(),
                    status: "active".to_string(),
                    status_changed_at: "2025-01-01T00:00:00Z".to_string(),
                    status_message: None,
                    updated_at: Some("2025-01-01T00:00:00Z".to_string()),
                }),
            }),
        }
    }

    #[tokio::test]
    async fn upsert_and_get_server() {
        let store = temp_store().await;

        let resp = make_server_response("io.github.user/weather", "1.0.0", true);
        let count = store.upsert_page(&[resp]).await.unwrap();
        assert_eq!(count, 1);

        let result = store
            .get_server("io.github.user/weather", "latest")
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, "1.0.0");
    }

    #[tokio::test]
    async fn get_server_by_version() {
        let store = temp_store().await;

        let resp = make_server_response("io.github.user/weather", "2.0.0", false);
        store.upsert_page(&[resp]).await.unwrap();

        let result = store
            .get_server("io.github.user/weather", "2.0.0")
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn get_server_not_found_returns_none() {
        let store = temp_store().await;

        let result = store.get_server("does.not/exist", "latest").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_versions_returns_all() {
        let store = temp_store().await;

        let r1 = make_server_response("io.github.user/weather", "1.0.0", false);
        let r2 = make_server_response("io.github.user/weather", "2.0.0", true);
        store.upsert_page(&[r1, r2]).await.unwrap();

        let versions = store
            .list_versions("io.github.user/weather")
            .await
            .unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[tokio::test]
    async fn list_servers_returns_paged() {
        let store = temp_store().await;

        let mut batch = Vec::new();
        for i in 0..5u32 {
            batch.push(make_server_response(
                &format!("io.github.user/server{i:02}"),
                "1.0.0",
                true,
            ));
        }
        store.upsert_page(&batch).await.unwrap();

        let params = StoreListParams { limit: Some(3), ..Default::default() };
        let page = store.list_servers(params).await.unwrap();
        assert_eq!(page.servers.len(), 3);
        assert!(page.next_cursor.is_some());

        // Fetch second page.
        let params2 = StoreListParams {
            limit: Some(3),
            cursor: page.next_cursor,
            ..Default::default()
        };
        let page2 = store.list_servers(params2).await.unwrap();
        assert_eq!(page2.servers.len(), 2);
        assert!(page2.next_cursor.is_none());
    }

    #[tokio::test]
    async fn list_servers_excludes_deleted_by_default() {
        let store = temp_store().await;
        use lab_apis::mcpregistry::types::{RegistryExtensions, ResponseMeta};

        // Insert one active, one deleted.
        let active = make_server_response("io.github.user/active", "1.0.0", true);
        let mut deleted = make_server_response("io.github.user/deleted", "1.0.0", false);
        deleted.meta = Some(ResponseMeta {
            official: Some(RegistryExtensions {
                is_latest: false,
                published_at: "2025-01-01T00:00:00Z".to_string(),
                status: "deleted".to_string(),
                status_changed_at: "2025-01-01T00:00:00Z".to_string(),
                status_message: None,
                updated_at: None,
            }),
        });

        store.upsert_page(&[active, deleted]).await.unwrap();

        let params = StoreListParams::default();
        let page = store.list_servers(params).await.unwrap();
        // Only the active server should appear.
        assert_eq!(page.servers.len(), 1);
        assert_eq!(page.servers[0].name, "io.github.user/active");
    }

    #[tokio::test]
    async fn update_is_latest_batch_runs() {
        let store = temp_store().await;

        let resp = make_server_response("io.github.user/weather", "1.0.0", true);
        store.upsert_page(&[resp]).await.unwrap();

        // Should not panic or error.
        store.update_is_latest().await.unwrap();

        let result = store
            .get_server("io.github.user/weather", "latest")
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn escape_like_escapes_special_chars() {
        assert_eq!(escape_like("a%b_c\\d"), r"a\%b\_c\\d");
    }
}
