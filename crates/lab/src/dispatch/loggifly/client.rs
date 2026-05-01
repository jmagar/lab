use lab_apis::loggifly::LoggiflyClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
///
/// Returns `None` only when the required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<LoggiflyClient> {
    Some(LoggiflyClient::with_local_paths(
        env_non_empty("LOGGIFLY_CONFIG_ROOT").map(Into::into),
        env_non_empty("LOGGIFLY_HEARTBEAT_PATH").map(Into::into),
        heartbeat_interval_secs(),
    ))
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<LoggiflyClient, ToolError> {
    Ok(client_from_env().unwrap_or_default())
}

/// Structured error returned when the required env vars are absent.
///
/// Exposed so API handlers can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "LoggiFly local heartbeat/config contract is unavailable".into(),
    }
}

fn heartbeat_interval_secs() -> Option<u64> {
    env_non_empty("LOGGIFLY_HEARTBEAT_INTERVAL_SECS")
        .or_else(|| env_non_empty("HEARTBEAT_INTERVAL"))
        .and_then(|value| value.parse::<u64>().ok())
}
