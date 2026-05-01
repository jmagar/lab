use lab_apis::navidrome::NavidromeClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<NavidromeClient> {
    let url = env_non_empty("NAVIDROME_URL")?;
    let username = env_non_empty("NAVIDROME_USERNAME")?;
    let token = env_non_empty("NAVIDROME_TOKEN")?;
    let salt = env_non_empty("NAVIDROME_SALT")?;
    NavidromeClient::with_subsonic_auth(&url, username, token, salt).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<NavidromeClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message:
            "NAVIDROME_URL, NAVIDROME_USERNAME, NAVIDROME_TOKEN, and NAVIDROME_SALT not configured"
                .into(),
    }
}
