use lab_apis::adguard::AdguardClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<AdguardClient> {
    let url = env_non_empty("ADGUARD_URL")?;
    if let Some(cookie) = env_non_empty("ADGUARD_SESSION_COOKIE") {
        AdguardClient::new(&url, Auth::Session { cookie }).ok()
    } else if let (Some(username), Some(password)) = (
        env_non_empty("ADGUARD_USERNAME"),
        env_non_empty("ADGUARD_PASSWORD"),
    ) {
        AdguardClient::with_login(&url, username, password).ok()
    } else {
        AdguardClient::new(&url, Auth::None).ok()
    }
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<AdguardClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "ADGUARD_URL not configured".into(),
    }
}
