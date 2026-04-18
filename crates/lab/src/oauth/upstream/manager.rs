//! Upstream OAuth lifecycle manager.
//!
//! `UpstreamOauthManager` orchestrates the full outbound `authorization_code` + PKCE
//! flow for one configured upstream MCP server.  It is per-upstream (constructed once
//! per `UpstreamConfig` that has an `oauth` block) and is `Clone` / `Send + Sync`.
//!
//! ## Subject
//!
//! All public methods take a `subject: &str` identifying the lab user initiating the
//! flow.  Credentials are stored and refreshed independently per `(upstream, subject)`.
//!
//! ## Two-phase authorization
//!
//! ```text
//! begin_authorization(subject)
//!   ↓  generates PKCE + CSRF, stores state in SQLite, returns redirect URL
//! browser → AS → callback
//!   ↓
//! complete_authorization_callback(subject, code, csrf)
//!   ↓  exchanges code, stores encrypted tokens in SQLite
//! build_auth_client(subject)
//!   ↓  loads stored credentials, proactively refreshes if stale
//! AuthClient<reqwest::Client>  → used by UpstreamPool for MCP calls
//! ```
//!
//! ## AS metadata caching
//!
//! Authorization server metadata is fetched once per upstream (not per-subject) and
//! cached to avoid an HTTP round-trip on every `build_auth_client` call.

use std::sync::Arc;

use dashmap::DashMap;
use lab_auth::sqlite::SqliteStore;
use rmcp::transport::auth::{AuthorizationMetadata, OAuthClientConfig};
use rmcp::transport::{AuthClient, AuthorizationManager};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::config::{UpstreamConfig, UpstreamOauthRegistration};
use crate::oauth::upstream::encryption::EncryptionKey;
use crate::oauth::upstream::refresh::{RefreshLocks, refresh_if_stale};
use crate::oauth::upstream::store::{SqliteCredentialStore, SqliteStateStore};
use crate::oauth::upstream::types::{BeginAuthorization, OauthError};

/// Upstream OAuth manager for a single upstream MCP server.
///
/// Cheap to clone — all mutable state is behind `Arc`.
#[derive(Clone)]
pub struct UpstreamOauthManager {
    sqlite: SqliteStore,
    key: EncryptionKey,
    upstream: UpstreamConfig,
    redirect_uri: Arc<String>,
    locks: Arc<RefreshLocks>,
    /// Cached AS metadata (fetched once per upstream, shared across subjects).
    metadata_cache: Arc<RwLock<Option<AuthorizationMetadata>>>,
    /// In-flight authorization managers keyed by subject (begin → complete window).
    pending: Arc<DashMap<String, Arc<Mutex<AuthorizationManager>>>>,
}

impl UpstreamOauthManager {
    /// Create a new manager for `upstream`.
    ///
    /// `redirect_uri` is the absolute URL of the OAuth callback endpoint that will
    /// receive the authorization code (e.g.
    /// `https://lab.example/v1/upstream-oauth/{name}/callback`).
    pub fn new(
        sqlite: SqliteStore,
        key: EncryptionKey,
        upstream: UpstreamConfig,
        redirect_uri: String,
    ) -> Self {
        Self {
            sqlite,
            key,
            upstream,
            redirect_uri: Arc::new(redirect_uri),
            locks: Arc::new(RefreshLocks::new()),
            metadata_cache: Arc::new(RwLock::new(None)),
            pending: Arc::new(DashMap::new()),
        }
    }

    /// Return `true` if persisted credentials exist for `subject`.
    ///
    /// Does not check whether the credentials are still valid.
    pub async fn has_credentials(&self, subject: &str) -> Result<bool, OauthError> {
        self.sqlite
            .find_upstream_oauth_credentials(&self.upstream.name, subject)
            .await
            .map(|opt| opt.is_some())
            .map_err(|e| OauthError::Internal(e.to_string()))
    }

    /// Begin the authorization flow.
    ///
    /// Discovers (or uses cached) AS metadata, registers or configures the OAuth
    /// client, generates a PKCE challenge, saves the pending state to SQLite, and
    /// returns the authorization URL to redirect the operator's browser to.
    ///
    /// Enforces S256 PKCE — returns `OauthError::UnsupportedMethod` if the AS does
    /// not advertise S256 in `code_challenge_methods_supported`.
    pub async fn begin_authorization(
        &self,
        subject: &str,
    ) -> Result<BeginAuthorization, OauthError> {
        let oauth_cfg = self.oauth_config()?;
        let upstream_url = self.upstream_url()?;

        let mut manager = AuthorizationManager::new(upstream_url.as_str())
            .await
            .map_err(|e| OauthError::Internal(format!("create auth manager: {e}")))?;

        let cred_store = SqliteCredentialStore::new(
            self.sqlite.clone(),
            self.key.clone(),
            &self.upstream.name,
            subject,
        );
        let state_store =
            SqliteStateStore::new(self.sqlite.clone(), &self.upstream.name, subject);
        manager.set_credential_store(cred_store);
        manager.set_state_store(state_store);

        let metadata = self.get_or_discover_metadata(&mut manager).await?;
        self.verify_s256(&metadata.code_challenge_methods_supported)?;
        manager.set_metadata(metadata);

        let scopes: Vec<&str> = oauth_cfg
            .scopes
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect();

        let client_cfg = self
            .resolve_client_config(&mut manager, &scopes)
            .await?;
        manager
            .configure_client(client_cfg)
            .map_err(|e| OauthError::Internal(format!("configure client: {e}")))?;

        let authorization_url = manager
            .get_authorization_url(&scopes)
            .await
            .map_err(|e| OauthError::Internal(format!("get authorization url: {e}")))?;

        self.pending
            .insert(subject.to_string(), Arc::new(Mutex::new(manager)));

        info!(
            upstream = %self.upstream.name,
            subject,
            "upstream oauth: authorization started"
        );

        Ok(BeginAuthorization { authorization_url })
    }

    /// Complete the authorization callback.
    ///
    /// Exchanges the authorization code for tokens, verifies the CSRF token, and
    /// persists the encrypted credentials.  Requires that `begin_authorization` was
    /// called first for this `(upstream, subject)` pair.
    pub async fn complete_authorization_callback(
        &self,
        subject: &str,
        code: &str,
        csrf_token: &str,
    ) -> Result<(), OauthError> {
        let manager_arc = self
            .pending
            .get(subject)
            .map(|v| v.clone())
            .ok_or_else(|| {
                OauthError::StateInvalid(
                    "no pending authorization; call begin_authorization first".to_string(),
                )
            })?;

        manager_arc
            .lock()
            .await
            .exchange_code_for_token(code, csrf_token)
            .await
            .map_err(map_auth_error)?;

        self.pending.remove(subject);

        info!(
            upstream = %self.upstream.name,
            subject,
            "upstream oauth: authorization completed"
        );

        Ok(())
    }

    /// Delete all stored credentials for `subject` and evict any cached state.
    pub async fn clear_credentials(&self, subject: &str) -> Result<(), OauthError> {
        self.sqlite
            .delete_upstream_oauth_credentials(&self.upstream.name, subject)
            .await
            .map_err(|e| OauthError::Internal(e.to_string()))?;

        self.pending.remove(subject);

        info!(
            upstream = %self.upstream.name,
            subject,
            "upstream oauth: credentials cleared"
        );

        Ok(())
    }

    /// Return an `AuthClient` ready for use, proactively refreshing if near expiry.
    ///
    /// Creates a fresh `AuthorizationManager` backed by stored credentials.  Uses
    /// cached AS metadata to avoid an extra HTTP round-trip.
    ///
    /// Returns `OauthError::NeedsReauth` when no credentials are stored or the
    /// refresh token has been revoked.
    pub async fn build_auth_client(
        &self,
        subject: &str,
    ) -> Result<AuthClient<reqwest::Client>, OauthError> {
        let upstream_url = self.upstream_url()?;

        let mut manager = AuthorizationManager::new(upstream_url.as_str())
            .await
            .map_err(|e| OauthError::Internal(format!("create auth manager: {e}")))?;

        let cred_store = SqliteCredentialStore::new(
            self.sqlite.clone(),
            self.key.clone(),
            &self.upstream.name,
            subject,
        );
        let state_store =
            SqliteStateStore::new(self.sqlite.clone(), &self.upstream.name, subject);
        manager.set_credential_store(cred_store);
        manager.set_state_store(state_store);

        let metadata = self.get_or_discover_metadata(&mut manager).await?;
        manager.set_metadata(metadata);

        let initialized = manager
            .initialize_from_store()
            .await
            .map_err(|e| OauthError::Internal(format!("initialize from store: {e}")))?;

        if !initialized {
            return Err(OauthError::NeedsReauth(format!(
                "no stored credentials for upstream '{}' subject '{subject}'",
                self.upstream.name
            )));
        }

        let manager_mutex = Mutex::new(manager);
        refresh_if_stale(
            &manager_mutex,
            &self.locks,
            &self.upstream.name,
            subject,
        )
        .await?;

        let manager = manager_mutex.into_inner();
        Ok(AuthClient::new(reqwest::Client::new(), manager))
    }

    // ---- private helpers ----

    fn oauth_config(&self) -> Result<&crate::config::UpstreamOauthConfig, OauthError> {
        self.upstream
            .oauth
            .as_ref()
            .ok_or_else(|| OauthError::Internal("upstream has no oauth config".to_string()))
    }

    fn upstream_url(&self) -> Result<Arc<String>, OauthError> {
        self.upstream
            .url
            .clone()
            .map(Arc::new)
            .ok_or_else(|| OauthError::Internal("upstream has no url".to_string()))
    }

    /// Fetch AS metadata, caching the result for subsequent calls.
    async fn get_or_discover_metadata(
        &self,
        manager: &mut AuthorizationManager,
    ) -> Result<AuthorizationMetadata, OauthError> {
        {
            let cached = self.metadata_cache.read().await;
            if let Some(meta) = cached.clone() {
                return Ok(meta);
            }
        }

        let metadata = manager
            .discover_metadata()
            .await
            .map_err(|e| OauthError::Internal(format!("discover metadata: {e}")))?;

        *self.metadata_cache.write().await = Some(metadata.clone());
        Ok(metadata)
    }

    fn verify_s256(&self, methods: &Option<Vec<String>>) -> Result<(), OauthError> {
        if let Some(methods) = methods {
            if !methods.iter().any(|m| m == "S256") {
                return Err(OauthError::UnsupportedMethod(format!(
                    "AS does not support S256 PKCE; advertised methods: {methods:?}"
                )));
            }
        } else {
            warn!(
                upstream = %self.upstream.name,
                "AS did not advertise code_challenge_methods_supported; proceeding with S256"
            );
        }
        Ok(())
    }

    async fn resolve_client_config(
        &self,
        manager: &mut AuthorizationManager,
        scopes: &[&str],
    ) -> Result<OAuthClientConfig, OauthError> {
        let oauth_cfg = self.oauth_config()?;
        match &oauth_cfg.registration {
            UpstreamOauthRegistration::Preregistered {
                client_id,
                client_secret_env,
            } => {
                let secret = client_secret_env
                    .as_deref()
                    .and_then(|var| std::env::var(var).ok())
                    .filter(|v| !v.is_empty());

                let mut cfg =
                    OAuthClientConfig::new(client_id.clone(), self.redirect_uri.as_str());
                if let Some(s) = secret {
                    cfg = cfg.with_client_secret(s);
                }
                cfg = cfg.with_scopes(scopes.iter().map(|s| s.to_string()).collect());
                Ok(cfg)
            }
            UpstreamOauthRegistration::Dynamic => manager
                .register_client("lab", self.redirect_uri.as_str(), scopes)
                .await
                .map_err(|e| OauthError::Internal(format!("dynamic registration: {e}"))),
            UpstreamOauthRegistration::ClientMetadataDocument { .. } => Err(OauthError::Internal(
                "ClientMetadataDocument registration is not yet supported".to_string(),
            )),
        }
    }
}

fn map_auth_error(e: rmcp::transport::AuthError) -> OauthError {
    match e {
        rmcp::transport::AuthError::AuthorizationRequired => {
            OauthError::NeedsReauth("authorization required".to_string())
        }
        rmcp::transport::AuthError::TokenExchangeFailed(msg) => OauthError::Internal(msg),
        rmcp::transport::AuthError::TokenRefreshFailed(msg) => {
            OauthError::NeedsReauth(format!("token refresh failed: {msg}"))
        }
        other => OauthError::Internal(other.to_string()),
    }
}
