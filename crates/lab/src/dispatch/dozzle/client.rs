use lab_apis::core::Auth;
use lab_apis::dozzle::DozzleClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<DozzleClient> {
    let url = env_non_empty("DOZZLE_URL")?;
    let auth = if let Some(cookie) = env_non_empty("DOZZLE_SESSION_COOKIE") {
        Auth::Session { cookie }
    } else {
        Auth::None
    };
    DozzleClient::new(&url, auth).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<DozzleClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "DOZZLE_URL not configured".into(),
    }
}
