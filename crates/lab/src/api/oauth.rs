//! OAuth 2.1 resource server support (RFC 9728).
//!
//! Lab acts as an OAuth resource server — it validates tokens, it does not
//! issue them. This module provides:
//! - `GET /.well-known/oauth-protected-resource` metadata endpoint
//! - `WWW-Authenticate` header generation for 401 responses
//! - `JwksManager` — JWKS cache with stale-while-revalidate
//! - `AuthContext` — extracted sub + scopes for downstream handlers
//!
//! The JWKS/JWT types are compiled now but first used in Phase 1.4
//! (JWT validation middleware).
// Narrow dead_code to items not yet wired at the call site.
// JwksManager and AuthContext are used starting in Phase 1.4.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{Json, extract::State, response::IntoResponse};
use jsonwebtoken::jwk::JwkSet;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};

use super::state::AppState;

/// RFC 9728 Protected Resource Metadata.
#[derive(Debug, Serialize)]
pub struct ProtectedResourceMetadata {
    /// The resource identifier (public URL of this lab instance).
    pub resource: String,
    /// Authorization servers this resource trusts.
    pub authorization_servers: Vec<String>,
    /// Scopes supported by this resource.
    pub scopes_supported: Vec<String>,
    /// Bearer methods supported.
    pub bearer_methods_supported: Vec<String>,
}

/// `GET /.well-known/oauth-protected-resource`
///
/// Returns RFC 9728 metadata so MCP clients can discover which auth server
/// to use. This endpoint is unauthenticated — clients need it before they
/// have a token.
pub async fn oauth_protected_resource(State(state): State<AppState>) -> impl IntoResponse {
    let resource_url = std::env::var("LAB_RESOURCE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://localhost".to_string());

    let issuer = std::env::var("LAB_OAUTH_ISSUER")
        .ok()
        .filter(|v| !v.is_empty());

    let authorization_servers = issuer.into_iter().collect();

    let _state = state;

    Json(ProtectedResourceMetadata {
        resource: resource_url,
        authorization_servers,
        scopes_supported: vec!["lab:read".to_string(), "lab:admin".to_string()],
        bearer_methods_supported: vec!["header".to_string()],
    })
}

/// Build the `WWW-Authenticate` header value for 401 responses.
pub fn www_authenticate_value() -> String {
    let resource_url = std::env::var("LAB_RESOURCE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://localhost".to_string());

    format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        resource_url.trim_end_matches('/')
    )
}

// ─── JWKS Manager ───────────────────────────────────────────────────────────

/// Error type for auth failures.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("JWKS refresh failed: {0}")]
    JwksRefreshFailed(String),
    #[error("OIDC discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("JWT validation failed: {0}")]
    ValidationFailed(String),
    #[error("missing kid in JWT header")]
    MissingKid,
    #[error("no matching key found for kid `{0}`")]
    UnknownKid(String),
}

/// Cached JWKS keys with TTL.
#[allow(dead_code)]
struct JwksCache {
    keys: JwkSet,
    fetched_at: Instant,
    ttl: Duration,
}

impl JwksCache {
    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > self.ttl
    }

    fn has_kid(&self, kid: &str) -> bool {
        self.keys
            .keys
            .iter()
            .any(|k| k.common.key_id.as_deref() == Some(kid))
    }
}

/// OIDC discovery response (minimal fields we need).
#[derive(Deserialize)]
struct OidcDiscovery {
    jwks_uri: String,
}

/// Authenticated caller context, injected as an axum extension.
///
/// Fields are constructed during JWT validation and available for
/// downstream scope checks. Not yet consumed — future handlers will
/// use `sub` for audit trails and `scopes` for scope-gated access.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    /// JWT `sub` claim — the authenticated user/client identifier.
    pub sub: String,
    /// Scopes granted by the token.
    pub scopes: Vec<String>,
    /// Token issuer.
    pub issuer: String,
}

/// JWT claims structure for token validation.
#[derive(Deserialize)]
struct JwtClaims {
    sub: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    scp: Option<Vec<String>>,
    iss: Option<String>,
}

/// JWKS key manager with stale-while-revalidate caching.
///
/// - Fetches JWKS from the issuer's OIDC discovery endpoint
/// - Caches keys with a configurable TTL (default: 1 hour)
/// - On `kid` mismatch, attempts eager refresh with thundering herd protection
/// - Never discards working keys on failed refresh (stale-while-revalidate)
pub struct JwksManager {
    issuer: String,
    audience: String,
    jwks_uri: String,
    cache: Arc<RwLock<JwksCache>>,
    refresh_semaphore: Arc<Semaphore>,
    http: reqwest::Client,
}

impl JwksManager {
    /// Create a new `JwksManager` by performing OIDC discovery.
    ///
    /// Fetches `{issuer}/.well-known/openid-configuration` to find the
    /// `jwks_uri`, then fetches the initial JWKS.
    pub async fn discover(issuer: &str, audience: &str) -> Result<Self, AuthError> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| AuthError::DiscoveryFailed(e.to_string()))?;

        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            issuer.trim_end_matches('/')
        );

        let discovery: OidcDiscovery = http
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| AuthError::DiscoveryFailed(format!("fetch {discovery_url}: {e}")))?
            .json()
            .await
            .map_err(|e| AuthError::DiscoveryFailed(format!("parse discovery: {e}")))?;

        let keys = Self::fetch_jwks_from(&http, &discovery.jwks_uri).await?;

        let cache = Arc::new(RwLock::new(JwksCache {
            keys,
            fetched_at: Instant::now(),
            ttl: Duration::from_secs(3600),
        }));

        Ok(Self {
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            jwks_uri: discovery.jwks_uri,
            cache,
            refresh_semaphore: Arc::new(Semaphore::new(1)),
            http,
        })
    }

    /// Ensure the cache contains a key with the given `kid`.
    ///
    /// On cache miss, attempts an eager refresh with thundering herd protection
    /// via double-checked locking on a semaphore.
    pub async fn ensure_kid(&self, kid: &str) -> Result<(), AuthError> {
        // Fast path: already cached
        if self.cache.read().await.has_kid(kid) {
            return Ok(());
        }
        // Serialize refreshes
        let _permit = self
            .refresh_semaphore
            .acquire()
            .await
            .map_err(|e| AuthError::JwksRefreshFailed(e.to_string()))?;
        // Recheck after acquiring (another task may have refreshed)
        if self.cache.read().await.has_kid(kid) {
            return Ok(());
        }
        // Fetch and install (stale-while-revalidate on failure)
        match Self::fetch_jwks_from(&self.http, &self.jwks_uri).await {
            Ok(new_keys) => {
                let mut cache = self.cache.write().await;
                cache.keys = new_keys;
                cache.fetched_at = Instant::now();
                Ok(())
            }
            Err(e) => {
                tracing::warn!(error = %e, kid, "JWKS refresh failed — serving stale keys");
                Err(AuthError::JwksRefreshFailed(e.to_string()))
            }
        }
    }

    /// Periodic refresh: if the cache TTL has expired, attempt a background
    /// refresh. Stale keys are kept on failure.
    #[allow(dead_code)] // Public API for future periodic JWKS refresh task
    pub async fn refresh_if_stale(&self) {
        let is_expired = self.cache.read().await.is_expired();
        if !is_expired {
            return;
        }
        let Ok(_permit) = self.refresh_semaphore.try_acquire() else {
            return;
        };
        if !self.cache.read().await.is_expired() {
            return;
        }
        match Self::fetch_jwks_from(&self.http, &self.jwks_uri).await {
            Ok(new_keys) => {
                let mut cache = self.cache.write().await;
                cache.keys = new_keys;
                cache.fetched_at = Instant::now();
                tracing::debug!("JWKS cache refreshed");
            }
            Err(e) => {
                tracing::warn!(error = %e, "JWKS background refresh failed — serving stale keys");
            }
        }
    }

    /// Validate a JWT and return the extracted `AuthContext`.
    pub async fn validate_jwt(&self, token: &str) -> Result<AuthContext, AuthError> {
        use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};

        let header = decode_header(token)
            .map_err(|e| AuthError::ValidationFailed(format!("decode header: {e}")))?;

        let kid = header.kid.ok_or(AuthError::MissingKid)?;

        // Ensure we have the key (may trigger eager refresh)
        if let Err(e) = self.ensure_kid(&kid).await {
            tracing::debug!(kid, error = %e, "JWKS eager refresh for kid failed — will try cached keys");
        }

        let cache = self.cache.read().await;
        let jwk = cache
            .keys
            .keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(&kid))
            .ok_or_else(|| AuthError::UnknownKid(kid.clone()))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AuthError::ValidationFailed(format!("create decoding key: {e}")))?;

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.algorithms = vec![
            Algorithm::RS256,
            Algorithm::RS384,
            Algorithm::RS512,
            Algorithm::ES256,
            Algorithm::ES384,
        ];

        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::ValidationFailed(e.to_string()))?;

        let claims = token_data.claims;

        let scopes = if let Some(scope_str) = &claims.scope {
            scope_str.split_whitespace().map(String::from).collect()
        } else {
            claims.scp.unwrap_or_default()
        };

        Ok(AuthContext {
            sub: claims.sub.unwrap_or_default(),
            scopes,
            issuer: claims.iss.unwrap_or_default(),
        })
    }

    async fn fetch_jwks_from(http: &reqwest::Client, jwks_uri: &str) -> Result<JwkSet, AuthError> {
        http.get(jwks_uri)
            .send()
            .await
            .map_err(|e| AuthError::JwksRefreshFailed(format!("fetch {jwks_uri}: {e}")))?
            .json()
            .await
            .map_err(|e| AuthError::JwksRefreshFailed(format!("parse JWKS: {e}")))
    }
}
