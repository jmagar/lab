use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::dispatch::error::ToolError;

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = std::env::var("UNIFI_URL").ok().filter(|v| !v.is_empty())?;
    let key = std::env::var("UNIFI_API_KEY").ok().filter(|v| !v.is_empty())?;
    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .ok()
}

pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}
