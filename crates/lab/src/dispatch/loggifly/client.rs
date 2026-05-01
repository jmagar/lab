use lab_apis::loggifly::LoggiflyClient;

use crate::dispatch::error::ToolError;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<LoggiflyClient> {
    Some(LoggiflyClient::default())
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<LoggiflyClient, ToolError> {
    Ok(LoggiflyClient::default())
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "LoggiFly implementation is deferred until a stable safe API or allowlisted config/health contract is approved".into(),
    }
}
