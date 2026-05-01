use lab_apis::freshrss::FreshrssClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<FreshrssClient> {
    let url = env_non_empty("FRESHRSS_URL")?;
    let username = env_non_empty("FRESHRSS_USERNAME")?;
    let password = env_non_empty("FRESHRSS_API_PASSWORD")?;
    FreshrssClient::with_credentials(&url, username, password).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<FreshrssClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "FRESHRSS_URL, FRESHRSS_USERNAME, and FRESHRSS_API_PASSWORD not configured".into(),
    }
}
