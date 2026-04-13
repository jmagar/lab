use lab_apis::core::Auth;
use lab_apis::prowlarr::ProwlarrClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Prowlarr` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
pub fn client_from_env() -> Option<ProwlarrClient> {
    let url = env_non_empty("PROWLARR_URL")?;
    let key = env_non_empty("PROWLARR_API_KEY")?;
    ProwlarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .map_err(|e| tracing::warn!(error = %e, url, "prowlarr client construction failed"))
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<ProwlarrClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Prowlarr env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<ProwlarrClient>` (e.g. the API
/// handler) can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "PROWLARR_URL or PROWLARR_API_KEY not configured".to_string(),
    }
}
