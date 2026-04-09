//! Shared dispatch helpers used across all service modules.

use serde_json::Value;

use crate::services::error::ToolError;

/// Serialize any `Serialize` value to `serde_json::Value`.
pub fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Extract a required string parameter from a JSON object.
pub fn require_str(params: &Value, key: &str) -> Result<String, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Clone a JSON object, stripping the given keys.
pub fn object_without(params: &Value, strip: &[&str]) -> Value {
    let mut map = params.as_object().cloned().unwrap_or_default();
    for key in strip {
        map.remove(*key);
    }
    Value::Object(map)
}

/// Build a request body from params.
///
/// If `params` contains a `payload` or `body` key, that value is used as the
/// body (parsed from a JSON string if necessary). Otherwise the entire params
/// object is used, minus the `strip` keys.
pub fn body_from_params(params: &Value, strip: &[&str]) -> Value {
    for key in ["payload", "body"] {
        if let Some(value) = params.get(key) {
            match value {
                Value::Object(map) => return Value::Object(map.clone()),
                Value::String(raw) => {
                    if let Ok(parsed) = serde_json::from_str(raw) {
                        return parsed;
                    }
                }
                _ => {}
            }
        }
    }
    object_without(params, strip)
}
