use lab_apis::core::Auth;
use lab_apis::qdrant::QdrantClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn require_client() -> Result<QdrantClient, ToolError> {
    let url = env_non_empty("QDRANT_URL").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "QDRANT_URL not configured".into(),
    })?;
    let auth = env_non_empty("QDRANT_API_KEY").map_or(Auth::None, |key| Auth::ApiKey {
        header: "api-key".into(),
        key,
    });
    QdrantClient::new(&url, auth).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("qdrant client init failed: {e}"),
    })
}

pub fn client_from_env() -> Option<QdrantClient> {
    let url = env_non_empty("QDRANT_URL")?;
    let auth = env_non_empty("QDRANT_API_KEY").map_or(Auth::None, |key| Auth::ApiKey {
        header: "api-key".into(),
        key,
    });
    QdrantClient::new(&url, auth).ok()
}
