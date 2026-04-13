use lab_apis::linkding::LinkdingClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Linkding` client from the default-instance env vars.
///
/// Linkding uses `Authorization: Token <api_token>` auth.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<LinkdingClient> {
    let url = env_non_empty("LINKDING_URL")?;
    let token = env_non_empty("LINKDING_TOKEN")?;
    client_from_vars(Some(&url), Some(&token))
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
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<LinkdingClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "LINKDING_URL or LINKDING_TOKEN not configured".to_string(),
    })
}
