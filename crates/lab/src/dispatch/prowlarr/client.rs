use lab_apis::core::Auth;
use lab_apis::prowlarr::ProwlarrClient;

use crate::dispatch::error::ToolError;

/// Build a `Prowlarr` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
pub fn client_from_env() -> Option<ProwlarrClient> {
    let url = std::env::var("PROWLARR_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let key = std::env::var("PROWLARR_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
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
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "PROWLARR_URL or PROWLARR_API_KEY not configured".to_string(),
    })
}
