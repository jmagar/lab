use lab_apis::bytestash::ByteStashClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn require_client() -> Result<ByteStashClient, ToolError> {
    let url = env_non_empty("BYTESTASH_URL").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "BYTESTASH_URL not configured".to_string(),
    })?;
    let token = env_non_empty("BYTESTASH_TOKEN").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "BYTESTASH_TOKEN not configured".to_string(),
    })?;
    ByteStashClient::new(
        &url,
        Auth::ApiKey {
            header: "bytestashauth".into(),
            key: format!("Bearer {token}"),
        },
    )
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("failed to initialize ByteStash client: {e}"),
    })
}

/// Build a `ByteStash` client from the default-instance env vars.
///
/// `ByteStash` uses a non-standard auth header: `bytestashauth: Bearer <jwt>`.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<ByteStashClient> {
    let url = env_non_empty("BYTESTASH_URL")?;
    let token = env_non_empty("BYTESTASH_TOKEN")?;
    client_from_vars(Some(&url), Some(&token))
}

/// Build a client from explicit URL and token values.
///
/// Returns `None` if either value is `None` or empty.
pub fn client_from_vars(url: Option<&str>, token: Option<&str>) -> Option<ByteStashClient> {
    let url = url.filter(|v| !v.is_empty())?;
    let token = token.filter(|v| !v.is_empty())?;
    ByteStashClient::new(
        url,
        Auth::ApiKey {
            header: "bytestashauth".into(),
            key: format!("Bearer {token}"),
        },
    )
    .ok()
}
