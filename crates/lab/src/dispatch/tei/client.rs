use lab_apis::core::Auth;
use lab_apis::tei::TeiClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn require_client() -> Result<TeiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "TEI_URL not configured".into(),
    })
}

pub fn client_from_env() -> Option<TeiClient> {
    let url = env_non_empty("TEI_URL")?;
    let auth = match env_non_empty("TEI_API_KEY") {
        Some(token) => Auth::Bearer { token },
        None => Auth::None,
    };
    TeiClient::new(&url, auth).ok()
}
