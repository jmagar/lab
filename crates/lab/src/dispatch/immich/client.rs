use lab_apis::core::Auth;
use lab_apis::immich::ImmichClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<ImmichClient> {
    let url = env_non_empty("IMMICH_URL")?;
    let key = env_non_empty("IMMICH_API_KEY")?;
    let auth = Auth::ApiKey {
        header: "x-api-key".into(),
        key,
    };
    ImmichClient::new(&url, auth).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<ImmichClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "IMMICH_URL and IMMICH_API_KEY not configured".into(),
    }
}
