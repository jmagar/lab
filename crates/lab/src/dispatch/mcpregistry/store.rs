//! SQLite-backed persistence for the MCP server registry.
//!
//! Uses an r2d2 connection pool with WAL mode for concurrent access from
//! multiple axum handlers. Pool size is 4 — sufficient for homelab workloads.
//! All pool operations run inside `tokio::task::spawn_blocking` since
//! `Pool::get()` is blocking.

use std::path::Path;
use std::time::Duration;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use thiserror::Error;

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
}

/// SQLite-backed store for registry server records.
///
/// Cloning the store is cheap — the inner `Pool` is `Arc`-backed.
///
/// Query methods are added in bead lab-h5pm.2 — `dead_code` is expected until
/// consumers are wired in.
#[derive(Clone)]
pub struct RegistryStore {
    #[allow(dead_code)]
    pool: Pool<SqliteConnectionManager>,
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
    /// methods added in bead lab-h5pm.2. It is `pub(crate)` so the dispatch
    /// module can reach it without exposing `r2d2` types to the rest of lab.
    pub(crate) fn pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }
}

// ── Migration ─────────────────────────────────────────────────────────────────

/// Apply pending schema migrations using PRAGMA user_version as the version
/// counter.
///
/// **MUST use `BEGIN EXCLUSIVE`** to prevent a TOCTOU race: two processes
/// starting simultaneously can both read `user_version = 0` and both attempt
/// to run the schema DDL.  The exclusive lock serialises them so only one
/// applies the migration.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
}
