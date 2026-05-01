use lab_apis::pihole::PiholeClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<PiholeClient> {
    let url = env_non_empty("PIHOLE_URL")?;
    let password = env_non_empty("PIHOLE_PASSWORD")?;
    PiholeClient::new(&url, password, env_non_empty("PIHOLE_TOTP")).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<PiholeClient, ToolError> {
    let url = env_non_empty("PIHOLE_URL").ok_or_else(not_configured_error)?;
    let password = env_non_empty("PIHOLE_PASSWORD").ok_or_else(not_configured_error)?;
    PiholeClient::new(&url, password, env_non_empty("PIHOLE_TOTP")).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("PIHOLE client init failed: {e}"),
    })
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "PIHOLE_URL/PIHOLE_PASSWORD not configured".into(),
    }
}
