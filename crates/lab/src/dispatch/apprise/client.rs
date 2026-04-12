use lab_apis::apprise::AppriseClient;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn require_client() -> Result<AppriseClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "APPRISE_URL not configured".into(),
    })
}

pub fn client_from_env() -> Option<AppriseClient> {
    let url = env_non_empty("APPRISE_URL")?;
    let auth = match env_non_empty("APPRISE_TOKEN") {
        Some(token) => Auth::Bearer { token },
        None => Auth::None,
    };
    AppriseClient::new(&url, auth).ok()
}
