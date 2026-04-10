use lab_apis::bytestash::ByteStashClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;

pub fn require_client() -> Result<ByteStashClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "BYTESTASH_URL or BYTESTASH_TOKEN not configured".to_string(),
    })
}

/// Build a `ByteStash` client from the default-instance env vars.
///
/// `ByteStash` uses a non-standard auth header: `bytestashauth: Bearer <jwt>`.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<ByteStashClient> {
    let url = std::env::var("BYTESTASH_URL").ok();
    let token = std::env::var("BYTESTASH_TOKEN").ok();
    client_from_vars(url.as_deref(), token.as_deref())
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
