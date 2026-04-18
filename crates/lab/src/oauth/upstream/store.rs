//! SQLite-backed rmcp `CredentialStore` and `StateStore` adapters.
//!
//! Both adapters are per-`(upstream_name, subject)` — construct a fresh pair for each
//! upstream × subject combination.  The underlying `SqliteStore` is `Clone` so the
//! shared connection pool is cheap to duplicate.
//!
//! # `StateStore::load` is a consuming take
//!
//! `StateStore::load` calls `take_upstream_oauth_state` (DELETE … RETURNING) rather than
//! a plain SELECT.  This is intentional: consuming state on first read closes the replay
//! window.  `StateStore::delete` is therefore a no-op — the row is already gone after
//! a successful `load`.
//!
//! # Lifetime pattern
//!
//! `#[async_trait]` expands `async fn foo(&self)` to a two-lifetime form:
//! `fn foo<'life0, 'async_trait>(&'life0 self) where 'life0: 'async_trait, Self: 'async_trait`.
//! All method impls below match that exact pattern without importing the `async-trait` crate.

use std::future::Future;
use std::pin::Pin;

use lab_auth::sqlite::SqliteStore;
use lab_auth::types::{UpstreamOauthCredentialRow, UpstreamOauthStateRow};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse as _};
use rmcp::transport::auth::{
    AuthError, CredentialStore, StateStore, StoredAuthorizationState, StoredCredentials,
};

use crate::oauth::upstream::encryption::{self, EncryptionKey};

fn credential_aad(upstream_name: &str, subject: &str, client_id: &str) -> Vec<u8> {
    format!("upstream={upstream_name}\0subject={subject}\0client_id={client_id}").into_bytes()
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Per-`(upstream_name, subject)` credential store backed by SQLite.
///
/// Tokens are encrypted at rest with ChaCha20-Poly1305.  Decryption failure
/// surfaces as `AuthError::AuthorizationRequired` so callers re-initiate the
/// authorization flow rather than hard-failing.
pub struct SqliteCredentialStore {
    store: SqliteStore,
    key: EncryptionKey,
    upstream_name: String,
    subject: String,
}

impl SqliteCredentialStore {
    pub fn new(
        store: SqliteStore,
        key: EncryptionKey,
        upstream_name: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            store,
            key,
            upstream_name: upstream_name.into(),
            subject: subject.into(),
        }
    }
}

impl CredentialStore for SqliteCredentialStore {
    fn load<'life0, 'async_trait>(
        &'life0 self,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<StoredCredentials>, AuthError>> + Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let row = self
                .store
                .find_upstream_oauth_credentials(&self.upstream_name, &self.subject)
                .await
                .map_err(|e| AuthError::InternalError(e.to_string()))?;

            let Some(row) = row else {
                return Ok(None);
            };

            let aad = credential_aad(&row.upstream_name, &row.subject, &row.client_id);
            let plaintext =
                encryption::open_with_aad(&self.key, &row.token_blob, &row.token_blob_nonce, &aad)
                    .map_err(|_| AuthError::AuthorizationRequired)?;

            let creds: StoredCredentials = serde_json::from_slice(&plaintext)
                .map_err(|e| AuthError::InternalError(format!("deserialize credentials: {e}")))?;

            Ok(Some(creds))
        })
    }

    fn save<'life0, 'async_trait>(
        &'life0 self,
        credentials: StoredCredentials,
    ) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let token_received_at = credentials.token_received_at.unwrap_or(0) as i64;

            let (access_token_expires_at, refresh_token_present) =
                if let Some(ref token) = credentials.token_response {
                    let expires_in = token.expires_in().map(|d| d.as_secs()).unwrap_or(3600) as i64;
                    (
                        token_received_at + expires_in,
                        token.refresh_token().is_some(),
                    )
                } else {
                    (0, false)
                };

            let granted_scopes_json = serde_json::to_string(&credentials.granted_scopes)
                .map_err(|e| AuthError::InternalError(format!("serialize scopes: {e}")))?;

            let plaintext = serde_json::to_vec(&credentials)
                .map_err(|e| AuthError::InternalError(format!("serialize credentials: {e}")))?;

            let aad = credential_aad(&self.upstream_name, &self.subject, &credentials.client_id);
            let (token_blob, token_blob_nonce) =
                encryption::seal_with_aad(&self.key, &plaintext, &aad);

            let row = UpstreamOauthCredentialRow {
                upstream_name: self.upstream_name.clone(),
                subject: self.subject.clone(),
                client_id: credentials.client_id.clone(),
                granted_scopes_json,
                token_blob,
                token_blob_nonce,
                token_received_at,
                access_token_expires_at,
                refresh_token_present,
            };

            self.store
                .upsert_upstream_oauth_credentials(row)
                .await
                .map_err(|e| AuthError::InternalError(e.to_string()))
        })
    }

    fn clear<'life0, 'async_trait>(
        &'life0 self,
    ) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            self.store
                .delete_upstream_oauth_credentials(&self.upstream_name, &self.subject)
                .await
                .map_err(|e| AuthError::InternalError(e.to_string()))
        })
    }
}

/// Per-`(upstream_name, subject)` state store backed by SQLite.
///
/// The `load` method uses `take_upstream_oauth_state` (atomic DELETE … RETURNING)
/// rather than a SELECT, consuming the row on first read.  `delete` is a no-op.
pub struct SqliteStateStore {
    store: SqliteStore,
    upstream_name: String,
    subject: String,
}

impl SqliteStateStore {
    pub fn new(
        store: SqliteStore,
        upstream_name: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            store,
            upstream_name: upstream_name.into(),
            subject: subject.into(),
        }
    }
}

/// TTL for pending authorization state (5 minutes).
const STATE_TTL_SECS: i64 = 300;

impl StateStore for SqliteStateStore {
    fn save<'life0, 'life1, 'async_trait>(
        &'life0 self,
        csrf_token: &'life1 str,
        state: StoredAuthorizationState,
    ) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let now = now_unix();
            let row = UpstreamOauthStateRow {
                upstream_name: self.upstream_name.clone(),
                subject: self.subject.clone(),
                csrf_token: csrf_token.to_string(),
                pkce_verifier: state.pkce_verifier,
                created_at: now,
                expires_at: now + STATE_TTL_SECS,
            };
            self.store
                .save_upstream_oauth_state(row)
                .await
                .map_err(|e| AuthError::InternalError(e.to_string()))
        })
    }

    fn load<'life0, 'life1, 'async_trait>(
        &'life0 self,
        csrf_token: &'life1 str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<StoredAuthorizationState>, AuthError>>
                + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let now = now_unix();
            let row = self
                .store
                .take_upstream_oauth_state(&self.upstream_name, &self.subject, csrf_token, now)
                .await
                .map_err(|e| AuthError::InternalError(e.to_string()))?;

            Ok(row.map(|r| {
                StoredAuthorizationState::new(
                    &PkceCodeVerifier::new(r.pkce_verifier),
                    &CsrfToken::new(r.csrf_token),
                )
            }))
        })
    }

    fn delete<'life0, 'life1, 'async_trait>(
        &'life0 self,
        _csrf_token: &'life1 str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        // `load` already performs an atomic DELETE … RETURNING; a separate
        // delete call would be a double-delete with no effect.
        Box::pin(async move { Ok(()) })
    }
}
