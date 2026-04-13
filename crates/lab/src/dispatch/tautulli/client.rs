use lab_apis::tautulli::TautulliClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Tautulli` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<TautulliClient> {
    let url = env_non_empty("TAUTULLI_URL")?;
    let key = env_non_empty("TAUTULLI_API_KEY")?;
    TautulliClient::new(&url, key)
        .map_err(|e| tracing::warn!(error = %e, url, "tautulli client construction failed"))
        .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<TautulliClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Tautulli env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<TautulliClient>` (e.g. the API
/// handler) can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "TAUTULLI_URL or TAUTULLI_API_KEY not configured".to_string(),
    }
}
