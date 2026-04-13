use lab_apis::core::Auth;
use lab_apis::overseerr::OverseerrClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build an `Overseerr` client from the default-instance env vars.
///
/// Returns `None` if either env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<OverseerrClient> {
    let url = env_non_empty("OVERSEERR_URL")?;
    let key = env_non_empty("OVERSEERR_API_KEY")?;
    OverseerrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .ok()
}

/// Return a client or a structured error for the MCP/CLI code paths.
///
/// # Errors
/// Returns [`ToolError::Sdk`] with `sdk_kind = "internal_error"` when
/// `OVERSEERR_URL` is absent or client construction fails.
pub fn require_client() -> Result<OverseerrClient, ToolError> {
    let url = env_non_empty("OVERSEERR_URL").ok_or_else(not_configured_error)?;
    let key = env_non_empty("OVERSEERR_API_KEY").ok_or_else(not_configured_error)?;
    OverseerrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("Overseerr client init failed: {e}"),
    })
}

/// Structured error for callers that hold a pre-built `Option<OverseerrClient>`.
///
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "OVERSEERR_URL or OVERSEERR_API_KEY not configured".to_string(),
    }
}
