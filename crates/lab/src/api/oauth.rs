use axum::http::request::Parts;

/// Stored in request extensions by the HTTP auth middleware.
///
/// Downstream handlers can read this when they need caller identity or scope
/// checks, but not every route consumes it yet.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub sub: String,
    pub scopes: Vec<String>,
    pub issuer: String,
    pub via_session: bool,
    pub csrf_token: Option<String>,
    pub email: Option<String>,
}

pub fn www_authenticate_value(resource_url: &str) -> String {
    format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        resource_url.trim_end_matches('/')
    )
}

pub fn auth_context(parts: &Parts) -> Option<&AuthContext> {
    parts.extensions.get::<AuthContext>()
}
