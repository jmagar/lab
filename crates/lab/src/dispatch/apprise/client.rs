use lab_apis::apprise::AppriseClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "APPRISE_URL not configured".into(),
    }
}

pub fn require_client() -> Result<AppriseClient, ToolError> {
    let url = env_non_empty("APPRISE_URL").ok_or_else(not_configured_error)?;
    let auth = env_non_empty("APPRISE_TOKEN").map_or(Auth::None, |token| Auth::Bearer { token });
    AppriseClient::new(&url, auth).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("apprise client init failed: {e}"),
    })
}

pub fn client_from_env() -> Option<AppriseClient> {
    let url = env_non_empty("APPRISE_URL")?;
    let auth = env_non_empty("APPRISE_TOKEN").map_or(Auth::None, |token| Auth::Bearer { token });
    AppriseClient::new(&url, auth).ok()
}
