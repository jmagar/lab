use lab_apis::linkding::LinkdingClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;

/// Build a `Linkding` client from the default-instance env vars.
///
/// Linkding uses `Authorization: Token <api_token>` auth.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<LinkdingClient> {
    let url = std::env::var("LINKDING_URL").ok();
    let token = std::env::var("LINKDING_TOKEN").ok();
    client_from_vars(url.as_deref(), token.as_deref())
}

/// Build a client from explicit URL and token values.
///
/// Returns `None` if either value is `None` or empty.
pub fn client_from_vars(url: Option<&str>, token: Option<&str>) -> Option<LinkdingClient> {
    let url = url.filter(|v| !v.is_empty())?;
    let token = token.filter(|v| !v.is_empty())?;
    LinkdingClient::new(
        url,
        Auth::ApiKey {
            header: "Authorization".into(),
            key: format!("Token {token}"),
        },
    )
    .map_err(|e| tracing::warn!(error = %e, url, "linkding client construction failed"))
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<LinkdingClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "LINKDING_URL or LINKDING_TOKEN not configured".to_string(),
    })
}
