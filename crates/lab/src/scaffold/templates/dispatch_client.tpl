use lab_apis::core::Auth;
use lab_apis::{{service}}::{{Service}}Client;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a client from the default service env vars.
pub fn client_from_env() -> Option<{{Service}}Client> {
    let url = env_non_empty("{{SERVICE}}_URL")?;
    {{Service}}Client::new(&url, Auth::None).ok()
}

/// Return a client or a structured configuration error.
pub fn require_client() -> Result<{{Service}}Client, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error used when the scaffolded service is not configured.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "{{SERVICE}}_URL not configured".into(),
    }
}
