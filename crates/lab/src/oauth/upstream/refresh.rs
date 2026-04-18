//! Single-flight refresh coordination for upstream OAuth clients.
//!
//! `RefreshLocks` prevents concurrent callers for the same `(upstream, subject)` pair
//! from issuing simultaneous token refresh requests against the authorization server.
//! One caller wins the lock and executes `get_access_token()` (which internally handles
//! proactive refresh); all others wait and then return the already-refreshed token.
//!
//! **Scope:** This module handles *proactive* refresh triggered before making an MCP call.
//! Reactive 401-retry logic is wired in Task 4 (`dispatch/gateway/`).
//!
//! ## rmcp refresh semantics
//!
//! `AuthorizationManager::get_access_token()` refreshes the token when fewer than 30 s
//! remain before expiry.  It does **not** react to 401 responses from the resource server.
//! A 401 with a locally-still-valid token requires an explicit `refresh_token()` call
//! followed by a retry — that is the Task 4 responsibility.

use std::sync::Arc;

use dashmap::DashMap;
use rmcp::transport::{AuthError, AuthorizationManager};
use tokio::sync::Mutex;

use crate::oauth::upstream::types::OauthError;

/// Per-`(upstream_name, subject)` mutex pool.
///
/// Entries are created lazily on first access and are never removed (the number of
/// distinct `(upstream, subject)` pairs is bounded by the number of configured upstreams
/// times the number of users, which is small in a homelab context).
#[derive(Default)]
pub struct RefreshLocks(DashMap<(String, String), Arc<Mutex<()>>>);

impl RefreshLocks {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    /// Return the mutex for `(upstream_name, subject)`, creating it if absent.
    pub fn acquire(&self, upstream_name: &str, subject: &str) -> Arc<Mutex<()>> {
        let key = (upstream_name.to_string(), subject.to_string());
        self.0
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

/// Proactively refresh the access token for `(upstream_name, subject)` if it is near
/// expiry, using a per-pair mutex to prevent concurrent refresh storms.
///
/// Returns the current access token (possibly just refreshed).
///
/// On `AuthError::AuthorizationRequired` (no credentials, or refresh token rejected),
/// maps to `OauthError::NeedsReauth` so callers can surface a re-authorization prompt.
pub async fn refresh_if_stale(
    auth_manager: &Mutex<AuthorizationManager>,
    locks: &RefreshLocks,
    upstream_name: &str,
    subject: &str,
) -> Result<String, OauthError> {
    let lock = locks.acquire(upstream_name, subject);
    let _guard = lock.lock().await;

    auth_manager
        .lock()
        .await
        .get_access_token()
        .await
        .map_err(|e| match e {
            AuthError::AuthorizationRequired => {
                OauthError::NeedsReauth("access token expired and refresh failed".to_string())
            }
            other => OauthError::Internal(other.to_string()),
        })
}
