use lab_apis::core::Auth;
use lab_apis::paperless::PaperlessClient;

use crate::dispatch::error::ToolError;

/// Build a `Paperless-ngx` client from the default-instance env vars.
///
/// Paperless-ngx uses token auth: `Authorization: Token <token>`.
/// Returns `None` if any required env var is absent or empty.
///
/// Called by `AppState::ServiceClients::from_env()` at startup.
#[allow(dead_code)]
pub fn client_from_env() -> Option<PaperlessClient> {
    let url = std::env::var("PAPERLESS_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let token = std::env::var("PAPERLESS_TOKEN")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("PAPERLESS_API_KEY").ok().filter(|v| !v.is_empty()))?;
    PaperlessClient::new(
        &url,
        Auth::ApiKey {
            header: "Authorization".into(),
            key: format!("Token {token}"),
        },
    )
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<PaperlessClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "PAPERLESS_URL or PAPERLESS_TOKEN (or PAPERLESS_API_KEY) not configured".to_string(),
    })
}
