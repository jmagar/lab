use lab_apis::sabnzbd::SabnzbdClient;

use crate::dispatch::error::ToolError;

/// Build a `SABnzbd` client from default env vars.
pub fn client_from_env() -> Option<SabnzbdClient> {
    let url = std::env::var("SABNZBD_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let key = std::env::var("SABNZBD_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
    SabnzbdClient::new(&url, key).ok()
}

pub fn require_client() -> Result<SabnzbdClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "SABNZBD_URL or SABNZBD_API_KEY not configured".to_string(),
    })
}
