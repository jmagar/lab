use lab_apis::core::Auth;
use lab_apis::plex::PlexClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Plex` client from the default-instance env vars.
///
/// Plex uses `X-Plex-Token` header auth.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<PlexClient> {
    let url = env_non_empty("PLEX_URL")?;
    let token = env_non_empty("PLEX_TOKEN")?;
    PlexClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Plex-Token".into(),
            key: token,
        },
    )
    .map_err(|e| tracing::warn!(error = %e, url, "plex client construction failed"))
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<PlexClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Plex env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<PlexClient>` (e.g. the API
/// handler) can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "PLEX_URL or PLEX_TOKEN not configured".to_string(),
    }
}
