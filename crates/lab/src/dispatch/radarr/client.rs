use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;

use crate::dispatch::error::ToolError;

/// Build a Radarr client from the default-instance env vars.
pub fn client_from_env() -> Option<RadarrClient> {
    let url = std::env::var("RADARR_URL").ok().filter(|v| !v.is_empty())?;
    let key = std::env::var("RADARR_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
    RadarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .ok()
}

pub fn require_client() -> Result<RadarrClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "RADARR_URL or RADARR_API_KEY not configured".to_string(),
    })
}
