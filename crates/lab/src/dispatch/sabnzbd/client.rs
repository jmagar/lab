use lab_apis::sabnzbd::SabnzbdClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `SABnzbd` client from default env vars.
pub fn client_from_env() -> Option<SabnzbdClient> {
    let url = env_non_empty("SABNZBD_URL")?;
    let key = env_non_empty("SABNZBD_API_KEY")?;
    SabnzbdClient::new(&url, key)
        .map_err(|e| tracing::warn!(error = %e, url, "sabnzbd client construction failed"))
        .ok()
}

pub fn require_client() -> Result<SabnzbdClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "SABNZBD_URL or SABNZBD_API_KEY not configured".to_string(),
    })
}
