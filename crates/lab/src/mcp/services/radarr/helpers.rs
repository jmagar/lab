//! Shared helpers for Radarr dispatch sub-modules.

use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use crate::services::error::ToolError;

pub fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Build a Radarr client from the default-instance env vars.
pub fn client_from_env() -> Option<RadarrClient> {
    let url = std::env::var("RADARR_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
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

pub fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

pub fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}
