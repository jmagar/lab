use lab_apis::core::Auth;
use lab_apis::openai::OpenAiClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Default `OpenAI` base URL.
const DEFAULT_BASE_URL: &str = "https://api.openai.com";

/// Build an `OpenAiClient` from the default-instance env vars.
///
/// Returns `None` if the required `OPENAI_API_KEY` env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<OpenAiClient> {
    let key = env_non_empty("OPENAI_API_KEY")?;
    let base_url = env_non_empty("OPENAI_URL")
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    OpenAiClient::new(&base_url, Auth::Bearer { token: key }).ok()
}

/// Return a client or a structured error.
///
/// # Errors
/// Returns `ToolError::Sdk` with `sdk_kind = "internal_error"` when:
/// - `OPENAI_API_KEY` is absent (not configured)
/// - TLS init fails
pub fn require_client() -> Result<OpenAiClient, ToolError> {
    let key = env_non_empty("OPENAI_API_KEY").ok_or_else(not_configured_error)?;
    let base_url = env_non_empty("OPENAI_URL")
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    OpenAiClient::new(&base_url, Auth::Bearer { token: key }).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("openai client init failed: {e}"),
    })
}

/// Structured error for callers holding a pre-built `Option<OpenAiClient>`.
/// API handlers call this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "OPENAI_API_KEY not configured".to_string(),
    }
}
