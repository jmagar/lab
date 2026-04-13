use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a Radarr client from the default-instance env vars.
pub fn client_from_env() -> Option<RadarrClient> {
    let url = env_non_empty("RADARR_URL")?;
    let key = env_non_empty("RADARR_API_KEY")?;
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
    let url = env_non_empty("RADARR_URL").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "RADARR_URL not configured".to_string(),
    })?;
    let key = env_non_empty("RADARR_API_KEY").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "RADARR_API_KEY not configured".to_string(),
    })?;
    RadarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    )
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("failed to initialize Radarr client: {e}"),
    })
}
