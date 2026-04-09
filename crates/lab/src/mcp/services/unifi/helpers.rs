//! Shared helpers for UniFi dispatch sub-modules.

use serde_json::{Map, Value};

use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::services::error::ToolError;

pub fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = std::env::var("UNIFI_URL").ok()?;
    let key = std::env::var("UNIFI_API_KEY").ok()?;
    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .ok()
}

pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
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

pub fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

pub fn require_u32(params: &Value, key: &str) -> Result<u32, ToolError> {
    let v = require_i64(params, key)?;
    u32::try_from(v).map_err(|_| ToolError::InvalidParam {
        message: format!("parameter `{key}` must be a non-negative integer"),
        param: key.to_string(),
    })
}

pub fn object_without(params: &Value, excluded: &[&str]) -> Result<Value, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let filtered: Map<String, Value> = obj
        .iter()
        .filter(|(k, _)| !excluded.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    Ok(Value::Object(filtered))
}

pub fn query_from(params: &Value, keys: &[&str]) -> Result<Vec<(String, String)>, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let mut out = Vec::new();
    for key in keys {
        if let Some(v) = obj.get(*key) {
            let rendered = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            out.push(((*key).to_string(), rendered));
        }
    }
    Ok(out)
}
