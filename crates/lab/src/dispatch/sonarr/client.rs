//! Client resolution for the `Sonarr` dispatch layer.
//!
//! Reads env vars and constructs `SonarrClient` instances. All env access lives
//! here — never in `dispatch.rs` or `params.rs`.

use lab_apis::core::Auth;
use lab_apis::sonarr::SonarrClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Sonarr` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<SonarrClient> {
    let url = env_non_empty("SONARR_URL")?;
    let key = env_non_empty("SONARR_API_KEY")?;
    SonarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .map_err(|e| tracing::warn!(error = %e, url, "sonarr client construction failed"))
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<SonarrClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Sonarr env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<SonarrClient>` (e.g. the
/// API handler) can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "SONARR_URL or SONARR_API_KEY not configured".to_string(),
    }
}
