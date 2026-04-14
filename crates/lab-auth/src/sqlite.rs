use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, params};

use crate::error::AuthError;
use crate::types::{
    AuthorizationCodeRow, AuthorizationRequestRow, RefreshTokenRow, RegisteredClient,
};
use crate::util::{ensure_restrictive_permissions, now_unix, set_restrictive_permissions};

const SQLITE_BUSY_TIMEOUT_MS: u64 = 5_000;

#[derive(Clone, Debug)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub async fn open(path: PathBuf) -> Result<Self, AuthError> {
        let conn = tokio::task::spawn_blocking(move || open_connection(path)).await;
        let store = match conn {
            Ok(result) => result,
            Err(error) => Err(AuthError::Storage(format!(
                "sqlite open task failed: {error}"
            ))),
        }
        .map(|conn| Self {
            conn: Arc::new(Mutex::new(conn)),
        })?;

        store.cleanup_expired().await?;
        Ok(store)
    }

    pub async fn pragma(&self, name: &str) -> Result<String, AuthError> {
        let pragma = match name {
            "journal_mode" | "busy_timeout" | "foreign_keys" => name.to_string(),
            other => {
                return Err(AuthError::Config(format!(
                    "unsupported pragma query `{other}`"
                )));
            }
        };

        self.with_conn(move |conn| {
            conn.query_row(&format!("PRAGMA {pragma};"), [], |row| {
                row.get::<_, Value>(0)
            })
            .map(|value| match value {
                Value::Text(text) => text,
                Value::Integer(int) => int.to_string(),
                other => format!("{other:?}"),
            })
            .map_err(sqlite_error)
        })
        .await
    }

    pub async fn register_client(&self, client: RegisteredClient) -> Result<(), AuthError> {
        self.with_conn(move |conn| {
            let redirect_uris = serde_json::to_string(&client.redirect_uris)
                .map_err(|error| AuthError::Storage(format!("serialize redirect_uris: {error}")))?;
            conn.execute(
                "INSERT INTO registered_clients (client_id, redirect_uris, created_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(client_id) DO UPDATE SET
                    redirect_uris = excluded.redirect_uris,
                    created_at = excluded.created_at",
                params![client.client_id, redirect_uris, client.created_at],
            )
            .map_err(sqlite_error)?;
            Ok(())
        })
        .await
    }

    pub async fn find_client(
        &self,
        client_id: &str,
    ) -> Result<Option<RegisteredClient>, AuthError> {
        let client_id = client_id.to_string();
        self.with_conn(move |conn| {
            conn.query_row(
                "SELECT client_id, redirect_uris, created_at
                 FROM registered_clients
                 WHERE client_id = ?1",
                params![client_id],
                |row| {
                    let redirect_uris: String = row.get(1)?;
                    let redirect_uris = serde_json::from_str(&redirect_uris).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok(RegisteredClient {
                        client_id: row.get(0)?,
                        redirect_uris,
                        created_at: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(sqlite_error)
        })
        .await
    }

    pub async fn insert_authorization_request(
        &self,
        request: AuthorizationRequestRow,
    ) -> Result<(), AuthError> {
        self.with_conn(move |conn| {
            conn.execute(
                "INSERT INTO authorization_requests (
                    state, client_id, redirect_uri, client_state, scope, provider_code_verifier,
                    code_challenge, code_challenge_method, created_at, expires_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    request.state,
                    request.client_id,
                    request.redirect_uri,
                    request.client_state,
                    request.scope,
                    request.provider_code_verifier,
                    request.code_challenge,
                    request.code_challenge_method,
                    request.created_at,
                    request.expires_at,
                ],
            )
            .map_err(sqlite_error)?;
            Ok(())
        })
        .await
    }

    pub async fn take_authorization_request(
        &self,
        state: &str,
    ) -> Result<AuthorizationRequestRow, AuthError> {
        let state = state.to_string();
        let now = now_unix();
        self.with_conn(move |conn| {
            conn.query_row(
                "DELETE FROM authorization_requests
                 WHERE state = ?1
                   AND expires_at > ?2
                 RETURNING state, client_id, redirect_uri, client_state, scope, provider_code_verifier,
                           code_challenge, code_challenge_method, created_at, expires_at",
                params![state, now],
                row_to_authorization_request,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AuthError::InvalidGrant(
                    "authorization state is missing, expired, or already used".to_string(),
                ),
                other => sqlite_error(other),
            })
        })
        .await
    }

    pub async fn insert_auth_code(&self, code: AuthorizationCodeRow) -> Result<(), AuthError> {
        self.with_conn(move |conn| {
            conn.execute(
                "INSERT INTO authorization_codes (
                    code, client_id, subject, redirect_uri, scope,
                    code_challenge, code_challenge_method, provider_refresh_token,
                    created_at, expires_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    code.code,
                    code.client_id,
                    code.subject,
                    code.redirect_uri,
                    code.scope,
                    code.code_challenge,
                    code.code_challenge_method,
                    code.provider_refresh_token,
                    code.created_at,
                    code.expires_at,
                ],
            )
            .map_err(sqlite_error)?;
            Ok(())
        })
        .await
    }

    pub async fn redeem_auth_code(&self, code: &str) -> Result<AuthorizationCodeRow, AuthError> {
        let code = code.to_string();
        let now = now_unix();
        self.with_conn(move |conn| {
            conn.query_row(
                "DELETE FROM authorization_codes
                 WHERE code = ?1
                   AND expires_at > ?2
                 RETURNING code, client_id, subject, redirect_uri, scope,
                           code_challenge, code_challenge_method, provider_refresh_token,
                           created_at, expires_at",
                params![code, now],
                row_to_authorization_code,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AuthError::InvalidGrant(
                    "authorization code is missing, expired, or already redeemed".to_string(),
                ),
                other => sqlite_error(other),
            })
        })
        .await
    }

    pub async fn upsert_refresh_token(&self, token: RefreshTokenRow) -> Result<(), AuthError> {
        self.with_conn(move |conn| {
            conn.execute(
                "INSERT INTO refresh_tokens (
                    refresh_token, client_id, subject, scope,
                    provider_refresh_token, created_at, expires_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(refresh_token) DO UPDATE SET
                    client_id = excluded.client_id,
                    subject = excluded.subject,
                    scope = excluded.scope,
                    provider_refresh_token = excluded.provider_refresh_token,
                    created_at = excluded.created_at,
                    expires_at = excluded.expires_at",
                params![
                    token.refresh_token,
                    token.client_id,
                    token.subject,
                    token.scope,
                    token.provider_refresh_token,
                    token.created_at,
                    token.expires_at,
                ],
            )
            .map_err(sqlite_error)?;
            Ok(())
        })
        .await
    }

    pub async fn find_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Option<RefreshTokenRow>, AuthError> {
        let refresh_token = refresh_token.to_string();
        let now = now_unix();
        self.with_conn(move |conn| {
            conn.query_row(
                "SELECT refresh_token, client_id, subject, scope,
                        provider_refresh_token, created_at, expires_at
                 FROM refresh_tokens
                 WHERE refresh_token = ?1
                   AND expires_at > ?2",
                params![refresh_token, now],
                row_to_refresh_token,
            )
            .optional()
            .map_err(sqlite_error)
        })
        .await
    }

    /// Delete expired rows from `authorization_requests`, `authorization_codes`, and
    /// `refresh_tokens`. Returns the total number of deleted rows.
    pub async fn cleanup_expired(&self) -> Result<u64, AuthError> {
        let now = now_unix();
        self.with_conn(move |conn| {
            let mut total: u64 = 0;
            for table in [
                "authorization_requests",
                "authorization_codes",
                "refresh_tokens",
            ] {
                let deleted = conn
                    .execute(
                        &format!("DELETE FROM {table} WHERE expires_at <= ?1"),
                        params![now],
                    )
                    .map_err(sqlite_error)?;
                total += deleted as u64;
            }
            Ok(total)
        })
        .await
    }

    async fn with_conn<T, F>(&self, op: F) -> Result<T, AuthError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, AuthError> + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let guard = conn
                .lock()
                .map_err(|_| AuthError::Storage("sqlite mutex poisoned".to_string()))?;
            op(&guard)
        })
        .await
        .map_err(|error| AuthError::Storage(format!("sqlite task failed: {error}")))?
    }
}

fn open_connection(path: PathBuf) -> Result<Connection, AuthError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AuthError::Storage(format!(
                "create auth database directory `{}`: {error}",
                parent.display()
            ))
        })?;
    }

    let existed = path.exists();
    if existed {
        ensure_restrictive_permissions(&path)?;
    }

    let conn = Connection::open(&path).map_err(sqlite_error)?;
    conn.busy_timeout(std::time::Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS))
        .map_err(sqlite_error)?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(sqlite_error)?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(sqlite_error)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS registered_clients (
            client_id TEXT PRIMARY KEY,
            redirect_uris TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS authorization_requests (
            state TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            redirect_uri TEXT NOT NULL,
            client_state TEXT NOT NULL,
            scope TEXT NOT NULL,
            provider_code_verifier TEXT NOT NULL,
            code_challenge TEXT NOT NULL,
            code_challenge_method TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS authorization_codes (
            code TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            subject TEXT NOT NULL,
            redirect_uri TEXT NOT NULL,
            scope TEXT NOT NULL,
            code_challenge TEXT NOT NULL,
            code_challenge_method TEXT NOT NULL,
            provider_refresh_token TEXT,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            refresh_token TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            subject TEXT NOT NULL,
            scope TEXT NOT NULL,
            provider_refresh_token TEXT,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        );",
    )
    .map_err(sqlite_error)?;

    if !existed {
        set_restrictive_permissions(&path)?;
    }
    ensure_restrictive_permissions(&path)?;

    Ok(conn)
}

fn sqlite_error(error: rusqlite::Error) -> AuthError {
    AuthError::Storage(format!("sqlite error: {error}"))
}

fn row_to_authorization_request(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AuthorizationRequestRow> {
    Ok(AuthorizationRequestRow {
        state: row.get(0)?,
        client_id: row.get(1)?,
        redirect_uri: row.get(2)?,
        client_state: row.get(3)?,
        scope: row.get(4)?,
        provider_code_verifier: row.get(5)?,
        code_challenge: row.get(6)?,
        code_challenge_method: row.get(7)?,
        created_at: row.get(8)?,
        expires_at: row.get(9)?,
    })
}

fn row_to_authorization_code(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuthorizationCodeRow> {
    Ok(AuthorizationCodeRow {
        code: row.get(0)?,
        client_id: row.get(1)?,
        subject: row.get(2)?,
        redirect_uri: row.get(3)?,
        scope: row.get(4)?,
        code_challenge: row.get(5)?,
        code_challenge_method: row.get(6)?,
        provider_refresh_token: row.get(7)?,
        created_at: row.get(8)?,
        expires_at: row.get(9)?,
    })
}

fn row_to_refresh_token(row: &rusqlite::Row<'_>) -> rusqlite::Result<RefreshTokenRow> {
    Ok(RefreshTokenRow {
        refresh_token: row.get(0)?,
        client_id: row.get(1)?,
        subject: row.get(2)?,
        scope: row.get(3)?,
        provider_refresh_token: row.get(4)?,
        created_at: row.get(5)?,
        expires_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::types::{AuthorizationCodeRow, RefreshTokenRow};

    use crate::util::now_unix;

    use super::SqliteStore;

    #[tokio::test]
    async fn sqlite_store_enables_wal_and_busy_timeout() {
        let store = temp_store().await;
        assert_eq!(pragma(&store, "journal_mode").await, "wal");
        assert!(pragma_ms(&store, "busy_timeout").await >= 5_000);
    }

    #[tokio::test]
    async fn sqlite_store_redeems_auth_code_only_once_under_race() {
        let store = temp_store().await;
        store.insert_auth_code(sample_code()).await.unwrap();
        let (a, b) = tokio::join!(
            store.redeem_auth_code("code-123"),
            store.redeem_auth_code("code-123"),
        );
        assert!(a.is_ok() ^ b.is_ok(), "a={a:?} b={b:?}");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn sqlite_store_refuses_world_readable_database_file() {
        use std::os::unix::fs::PermissionsExt;

        let path = temp_db_path();
        std::fs::write(&path, []).unwrap();
        std::fs::set_permissions(&path, PermissionsExt::from_mode(0o644)).unwrap();
        let err = SqliteStore::open(path).await.unwrap_err();
        assert!(err.to_string().contains("permissions"));
    }

    #[tokio::test]
    async fn sqlite_store_rejects_expired_authorization_code() {
        let store = temp_store().await;
        let mut code = sample_code();
        code.expires_at = now_unix() - 1;
        store.insert_auth_code(code).await.unwrap();
        let err = store.redeem_auth_code("code-123").await.unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[tokio::test]
    async fn sqlite_store_ignores_expired_refresh_token() {
        let store = temp_store().await;
        store
            .upsert_refresh_token(RefreshTokenRow {
                refresh_token: "refresh-token".to_string(),
                client_id: "client".to_string(),
                subject: "google-user".to_string(),
                scope: "lab".to_string(),
                provider_refresh_token: Some("provider-refresh".to_string()),
                created_at: now_unix() - 300,
                expires_at: now_unix() - 1,
            })
            .await
            .unwrap();
        assert!(
            store
                .find_refresh_token("refresh-token")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn sqlite_store_cleanup_expired_removes_stale_rows() {
        let store = temp_store().await;
        let now = now_unix();

        // Insert an expired auth code.
        let mut code = sample_code();
        code.expires_at = now - 10;
        store.insert_auth_code(code).await.unwrap();

        // Insert an expired refresh token.
        store
            .upsert_refresh_token(RefreshTokenRow {
                refresh_token: "expired-refresh".to_string(),
                client_id: "client".to_string(),
                subject: "google-user".to_string(),
                scope: "lab".to_string(),
                provider_refresh_token: None,
                created_at: now - 600,
                expires_at: now - 10,
            })
            .await
            .unwrap();

        // Insert an expired authorization request.
        use crate::types::AuthorizationRequestRow;
        store
            .insert_authorization_request(AuthorizationRequestRow {
                state: "expired-state".to_string(),
                client_id: "client".to_string(),
                redirect_uri: "http://127.0.0.1:7777/callback".to_string(),
                client_state: "cs".to_string(),
                scope: "lab".to_string(),
                provider_code_verifier: "verifier".to_string(),
                code_challenge: "challenge".to_string(),
                code_challenge_method: "S256".to_string(),
                created_at: now - 600,
                expires_at: now - 10,
            })
            .await
            .unwrap();

        // Insert a valid (non-expired) refresh token.
        store
            .upsert_refresh_token(RefreshTokenRow {
                refresh_token: "valid-refresh".to_string(),
                client_id: "client".to_string(),
                subject: "google-user".to_string(),
                scope: "lab".to_string(),
                provider_refresh_token: None,
                created_at: now,
                expires_at: now + 3600,
            })
            .await
            .unwrap();

        let deleted = store.cleanup_expired().await.unwrap();
        assert_eq!(deleted, 3, "should delete exactly 3 expired rows");

        // The valid refresh token should still exist.
        assert!(
            store
                .find_refresh_token("valid-refresh")
                .await
                .unwrap()
                .is_some()
        );
    }

    async fn temp_store() -> SqliteStore {
        SqliteStore::open(temp_db_path()).await.unwrap()
    }

    async fn pragma(store: &SqliteStore, name: &str) -> String {
        store.pragma(name).await.unwrap()
    }

    async fn pragma_ms(store: &SqliteStore, name: &str) -> u64 {
        pragma(store, name).await.parse().unwrap()
    }

    fn temp_db_path() -> PathBuf {
        tempfile::tempdir().unwrap().keep().join("auth.db")
    }

    fn sample_code() -> AuthorizationCodeRow {
        let now = now_unix();
        AuthorizationCodeRow {
            code: "code-123".to_string(),
            client_id: "client".to_string(),
            subject: "google-user".to_string(),
            redirect_uri: "http://127.0.0.1:7777/callback".to_string(),
            scope: "lab".to_string(),
            code_challenge: "challenge".to_string(),
            code_challenge_method: "S256".to_string(),
            provider_refresh_token: Some("provider-refresh".to_string()),
            created_at: now,
            expires_at: now + 300,
        }
    }
}
