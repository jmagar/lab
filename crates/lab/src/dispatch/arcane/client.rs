use lab_apis::arcane::ArcaneClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build an `Arcane` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<ArcaneClient> {
    let url = env_non_empty("ARCANE_URL")?;
    let key = env_non_empty("ARCANE_API_KEY")?;
    ArcaneClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-Key".into(),
            key,
        },
    )
    .ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<ArcaneClient, ToolError> {
    let url = env_non_empty("ARCANE_URL").ok_or_else(not_configured_error)?;
    let key = env_non_empty("ARCANE_API_KEY").ok_or_else(not_configured_error)?;
    ArcaneClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-Key".into(),
            key,
        },
    )
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("Arcane client init failed: {e}"),
    })
}

/// Structured error for callers that hold a pre-built `Option<ArcaneClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "ARCANE_URL or ARCANE_API_KEY not configured".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_configured_error_has_expected_kind() {
        let err = not_configured_error();
        assert_eq!(err.kind(), "internal_error");
    }
}
