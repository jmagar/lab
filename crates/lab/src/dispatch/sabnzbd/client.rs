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
    let url = std::env::var("SABNZBD_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: "SABNZBD_URL not configured".to_string(),
        })?;
    let key = std::env::var("SABNZBD_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: "SABNZBD_API_KEY not configured".to_string(),
        })?;
    SabnzbdClient::new(&url, key).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("failed to initialize SABnzbd client: {e}"),
    })
}
