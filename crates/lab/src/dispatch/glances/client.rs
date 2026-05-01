use lab_apis::glances::GlancesClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<GlancesClient> {
    let url = env_non_empty("GLANCES_URL")?;
    let auth = if let Some(key) = env_non_empty("GLANCES_API_KEY") {
        lab_apis::core::Auth::ApiKey { header: "X-Api-Key".into(), key }
    } else if let Some(token) = env_non_empty("GLANCES_TOKEN") {
        lab_apis::core::Auth::Bearer { token }
    } else {
        lab_apis::core::Auth::None
    };
    GlancesClient::new(&url, auth).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<GlancesClient, ToolError> {
    let url = env_non_empty("GLANCES_URL").ok_or_else(not_configured_error)?;
    GlancesClient::new(&url, Auth::None).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("GLANCES client init failed: {e}"),
    })
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "GLANCES_URL not configured".into(),
    }
}
