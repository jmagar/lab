use lab_apis::core::Auth;
use lab_apis::mcpregistry::client::{McpRegistryClient, REGISTRY_DEFAULT_URL};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `McpRegistry` client from the default-instance env vars.
///
/// Returns `None` if the env var is absent or empty; falls back to the
/// public default URL when unset.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<McpRegistryClient> {
    let url = env_non_empty("MCPREGISTRY_URL").unwrap_or_else(|| REGISTRY_DEFAULT_URL.to_string());
    McpRegistryClient::new(&url, Auth::None).ok()
}

/// Return a client or a structured error.
///
/// Always succeeds unless TLS init fails; the registry URL defaults to the
/// public endpoint when `MCPREGISTRY_URL` is not set.
pub fn require_client() -> Result<McpRegistryClient, ToolError> {
    let url = env_non_empty("MCPREGISTRY_URL").unwrap_or_else(|| REGISTRY_DEFAULT_URL.to_string());
    McpRegistryClient::new(&url, Auth::None).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("mcpregistry client init failed: {e}"),
    })
}

/// Structured error for callers that hold a pre-built `Option<McpRegistryClient>`.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "mcpregistry client is not available".to_string(),
    }
}
