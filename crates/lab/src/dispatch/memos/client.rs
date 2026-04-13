use lab_apis::core::Auth;
use lab_apis::memos::MemosClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `Memos` client from the default-instance env vars.
///
/// Memos uses `Authorization: Bearer <token>` auth.
/// Returns `None` if any required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<MemosClient> {
    let url = env_non_empty("MEMOS_URL")?;
    let token = env_non_empty("MEMOS_TOKEN")?;
    client_from_vars(Some(&url), Some(&token))
}

/// Build a client from explicit URL and token values.
///
/// Returns `None` if either value is `None` or empty.
pub fn client_from_vars(url: Option<&str>, token: Option<&str>) -> Option<MemosClient> {
    let url = url.filter(|v| !v.is_empty())?;
    let token = token.filter(|v| !v.is_empty())?;
    MemosClient::new(url, Auth::Bearer {
        token: token.to_string(),
    })
    .map_err(|e| tracing::warn!(error = %e, url, "memos client construction failed"))
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<MemosClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Memos env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<MemosClient>` (e.g.
/// the API handler) can produce the same error without re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "MEMOS_URL or MEMOS_TOKEN not configured".to_string(),
    }
}
