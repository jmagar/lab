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

use lab_auth::sqlite::SqliteStore;
use lab_auth::types::UpstreamOauthCredentialRow;
use rmcp::transport::auth::{AuthorizationMetadata, OAuthClientConfig};
use rmcp::transport::{AuthClient, AuthorizationManager};
use tokio::sync::RwLock;
use tracing::info;

use crate::config::{UpstreamConfig, UpstreamOauthRegistration};
use crate::oauth::upstream::encryption::EncryptionKey;
use crate::oauth::upstream::refresh::RefreshLocks;
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
        let state_store = SqliteStateStore::new(self.sqlite.clone(), &self.upstream.name, subject);
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

        let client_cfg = self.resolve_client_config(&mut manager, &scopes).await?;
        manager
            .configure_client(client_cfg)
            .map_err(|e| OauthError::Internal(format!("configure client: {e}")))?;

        let authorization_url = manager
            .get_authorization_url(&scopes)
            .await
            .map_err(|e| OauthError::Internal(format!("get authorization url: {e}")))?;
        let _csrf = extract_state_param(&authorization_url).ok_or_else(|| {
            OauthError::Internal("authorization url missing required state parameter".to_string())
        })?;

        info!(
            upstream = %self.upstream.name,
            subject,
            "upstream oauth: authorization started"
        );

        Ok(BeginAuthorization { authorization_url })
    }

    /// Complete the authorization callback.
    ///
    /// Exchanges the authorization code for tokens and persists the encrypted
    /// credentials. Completion is reconstructed from persisted PKCE state rather
    /// than an in-memory pending map, so callbacks remain valid across restarts.
    pub async fn complete_authorization_callback(
        &self,
        subject: &str,
        code: &str,
        csrf_token: &str,
    ) -> Result<(), OauthError> {
        self.configured_authorization_manager(subject)
            .await?
            .exchange_code_for_token(code, csrf_token)
            .await
            .map_err(map_auth_error)?;

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
        let lock = self.locks.acquire(&self.upstream.name, subject);
        let _guard = lock.lock().await;

        let mut manager = self.configured_authorization_manager(subject).await?;
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

        manager.get_access_token().await.map_err(map_auth_error)?;
        Ok(AuthClient::new(reqwest::Client::new(), manager))
    }

    pub async fn credential_row(
        &self,
        subject: &str,
    ) -> Result<Option<UpstreamOauthCredentialRow>, OauthError> {
        self.sqlite
            .find_upstream_oauth_credentials(&self.upstream.name, subject)
            .await
            .map_err(|e| OauthError::Internal(e.to_string()))
    }

    #[allow(dead_code)]
    pub async fn subject_for_state(&self, csrf_token: &str) -> Result<Option<String>, OauthError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| OauthError::Internal(format!("system clock error: {error}")))?
            .as_secs() as i64;
        self.sqlite
            .find_upstream_oauth_state_subject(&self.upstream.name, csrf_token, now)
            .await
            .map_err(|e| OauthError::Internal(e.to_string()))
    }

    // ---- private helpers ----

    async fn configured_authorization_manager(
        &self,
        subject: &str,
    ) -> Result<AuthorizationManager, OauthError> {
        let upstream_url = self.upstream_url()?;
        let oauth_cfg = self.oauth_config()?;

        let mut manager = AuthorizationManager::new(upstream_url.as_str())
            .await
            .map_err(|e| OauthError::Internal(format!("create auth manager: {e}")))?;

        let cred_store = SqliteCredentialStore::new(
            self.sqlite.clone(),
            self.key.clone(),
            &self.upstream.name,
            subject,
        );
        let state_store = SqliteStateStore::new(self.sqlite.clone(), &self.upstream.name, subject);
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
        let client_cfg = self.resolve_client_config(&mut manager, &scopes).await?;
        manager
            .configure_client(client_cfg)
            .map_err(|e| OauthError::Internal(format!("configure client: {e}")))?;
        Ok(manager)
    }

    fn oauth_config(&self) -> Result<&crate::config::UpstreamOauthConfig, OauthError> {
        self.upstream
            .oauth
            .as_ref()
            .ok_or_else(|| OauthError::Internal("upstream has no oauth config".to_string()))
    }

    fn upstream_url(&self) -> Result<Arc<String>, OauthError> {
        let canonical = self
            .upstream
            .canonical_url()
            .ok_or_else(|| OauthError::Internal("upstream has no url".to_string()))?
            .map_err(|e| OauthError::Internal(format!("invalid upstream url: {e}")))?;
        Ok(Arc::new(canonical))
    }

    /// Fetch AS metadata, caching the result for subsequent calls.
    ///
    /// Enforces issuer binding per RFC 8414: `issuer` MUST be present and the
    /// `authorization_endpoint` + `token_endpoint` MUST share its origin. Rejects
    /// silent issuer drift between the first and subsequent discovery calls.
    ///
    /// Uses a single write-lock acquisition to avoid a TOCTOU race between a
    /// read-lock check and a subsequent write-lock cache update.
    async fn get_or_discover_metadata(
        &self,
        manager: &mut AuthorizationManager,
    ) -> Result<AuthorizationMetadata, OauthError> {
        let mut cache = self.metadata_cache.write().await;
        if let Some(meta) = cache.clone() {
            return Ok(meta);
        }

        let metadata = manager
            .discover_metadata()
            .await
            .map_err(|e| OauthError::Internal(format!("discover metadata: {e}")))?;

        self.verify_issuer_binding(&metadata)?;

        *cache = Some(metadata.clone());
        Ok(metadata)
    }

    /// RFC 8414 §3 issuer binding: `issuer` must be present, and every
    /// non-jwks endpoint origin (scheme + host + port) must match the
    /// issuer origin. This is stricter than a host-only check: it rejects
    /// endpoints served over a different scheme (e.g. http vs https) or
    /// on a different port, which a host-only comparison would miss.
    fn verify_issuer_binding(&self, metadata: &AuthorizationMetadata) -> Result<(), OauthError> {
        let issuer_raw = metadata.issuer.as_deref().ok_or_else(|| {
            OauthError::IssuerMismatch(format!(
                "AS metadata for upstream '{}' is missing required `issuer` claim",
                self.upstream.name
            ))
        })?;
        // Normalize the issuer: strip trailing slashes for a canonical form.
        let issuer_normalized = issuer_raw.trim_end_matches('/');
        let issuer_origin = url_origin(issuer_normalized).ok_or_else(|| {
            OauthError::IssuerMismatch(format!("issuer `{issuer_raw}` is not a valid URL"))
        })?;
        for (label, endpoint) in [
            (
                "authorization_endpoint",
                Some(metadata.authorization_endpoint.as_str()),
            ),
            ("token_endpoint", Some(metadata.token_endpoint.as_str())),
            (
                "registration_endpoint",
                metadata.registration_endpoint.as_deref(),
            ),
        ] {
            let Some(endpoint) = endpoint else { continue };
            let Some(origin) = url_origin(endpoint) else {
                return Err(OauthError::IssuerMismatch(format!(
                    "{label} `{endpoint}` is not a valid URL"
                )));
            };
            if origin != issuer_origin {
                return Err(OauthError::IssuerMismatch(format!(
                    "{label} origin `{origin}` does not match issuer origin `{issuer_origin}`"
                )));
            }
        }
        Ok(())
    }

    fn verify_s256(&self, methods: &Option<Vec<String>>) -> Result<(), OauthError> {
        match methods {
            Some(methods) if methods.iter().any(|m| m == "S256") => Ok(()),
            Some(methods) => Err(OauthError::UnsupportedMethod(format!(
                "AS does not advertise S256 PKCE; advertised methods: {methods:?}"
            ))),
            None => Err(OauthError::UnsupportedMethod(
                "AS did not advertise code_challenge_methods_supported; S256 is required"
                    .to_string(),
            )),
        }
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
                let secret = match client_secret_env.as_deref() {
                    None => None,
                    Some(var) => {
                        let val = std::env::var(var).unwrap_or_default();
                        if val.is_empty() {
                            return Err(OauthError::Internal(format!(
                                "client_secret_env '{var}' is configured but env var '{var}' is not set or is empty"
                            )));
                        }
                        Some(val)
                    }
                };

                let mut cfg = OAuthClientConfig::new(client_id.clone(), self.redirect_uri.as_str());
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
            UpstreamOauthRegistration::ClientMetadataDocument { url } => {
                // Client ID Metadata Document (CIMD): the metadata-document URL
                // *is* the client identifier. No registration_endpoint call is
                // issued — the AS fetches the document itself when it first sees
                // the client_id. We construct the OAuth client locally.
                url::Url::parse(url).map_err(|e| {
                    OauthError::Internal(format!("invalid client_metadata_document url: {e}"))
                })?;
                let cfg = OAuthClientConfig::new(url.clone(), self.redirect_uri.as_str())
                    .with_scopes(scopes.iter().map(|s| s.to_string()).collect());
                Ok(cfg)
            }
        }
    }
}

/// Return the normalized origin (scheme + "://" + lowercased host + optional explicit port)
/// of a URL, or `None` if the URL is invalid or has no host.
///
/// This is stricter than a host-only comparison: it rejects URLs that share a host
/// but differ in scheme or port (e.g. http vs https, or :80 vs :8080).
fn url_origin(s: &str) -> Option<String> {
    let u = url::Url::parse(s).ok()?;
    let host = u.host_str()?.to_ascii_lowercase();
    let scheme = u.scheme();
    match u.port() {
        Some(port) => Some(format!("{scheme}://{host}:{port}")),
        None => Some(format!("{scheme}://{host}")),
    }
}

/// Return the lowercased host of a URL, or `None` if the URL is invalid or hostless.
#[allow(dead_code)]
fn url_host(s: &str) -> Option<String> {
    url::Url::parse(s)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
}

fn extract_state_param(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
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
