use lab_apis::core::Auth;
use lab_apis::scrutiny::ScrutinyClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<ScrutinyClient> {
    let url = env_non_empty("SCRUTINY_URL")?;
    let auth = if let Some(key) = env_non_empty("SCRUTINY_API_KEY") {
        Auth::Bearer { token: key }
    } else if let Some(token) = env_non_empty("SCRUTINY_TOKEN") {
        Auth::Bearer { token }
    } else {
        Auth::None
    };
    ScrutinyClient::new(&url, auth).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<ScrutinyClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "SCRUTINY_URL not configured".into(),
    }
}
