use lab_apis::core::Auth;
use lab_apis::qdrant::QdrantClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn require_client() -> Result<QdrantClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "QDRANT_URL not configured".into(),
    })
}

pub fn client_from_env() -> Option<QdrantClient> {
    let url = env_non_empty("QDRANT_URL")?;
    let auth = match env_non_empty("QDRANT_API_KEY") {
        Some(key) => Auth::ApiKey {
            header: "api-key".into(),
            key,
        },
        None => Auth::None,
    };
    QdrantClient::new(&url, auth).ok()
}
