//! OAuth 2.1 resource server support (RFC 9728).
//!
//! Lab acts as an OAuth resource server — it validates tokens, it does not
//! issue them. This module provides:
//! - `GET /.well-known/oauth-protected-resource` metadata endpoint
//! - `WWW-Authenticate` header generation for 401 responses

use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

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
pub async fn oauth_protected_resource(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let resource_url = std::env::var("LAB_RESOURCE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://localhost".to_string());

    let issuer = std::env::var("LAB_OAUTH_ISSUER")
        .ok()
        .filter(|v| !v.is_empty());

    let authorization_servers = issuer.into_iter().collect();

    // Expose scopes via AppState if available in future; for now use the
    // canonical two-level scope set.
    let _ = &state;

    Json(ProtectedResourceMetadata {
        resource: resource_url,
        authorization_servers,
        scopes_supported: vec!["lab:read".to_string(), "lab:admin".to_string()],
        bearer_methods_supported: vec!["header".to_string()],
    })
}

/// Build the `WWW-Authenticate` header value for 401 responses.
///
/// Includes `resource_metadata` pointing to the discovery endpoint so MCP
/// clients can auto-discover the auth configuration.
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
