use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::helpers::{optional_str, optional_u32, require_str, to_json};

/// Extract a required i64 (signed integer) parameter from a JSON object.
pub fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract an optional i64 parameter from a JSON object.
pub fn optional_i64(params: &Value, key: &str) -> Result<Option<i64>, ToolError> {
    params.get(key).map_or(Ok(None), |v| {
        v.as_i64().map(Some).ok_or_else(|| ToolError::InvalidParam {
            message: format!("parameter `{key}` must be an integer"),
            param: key.to_string(),
        })
    })
}

/// Extract a required i32 (32-bit integer) parameter from a JSON object.
pub fn require_i32(params: &Value, key: &str) -> Result<i32, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract a required f64 (floating-point) parameter from a JSON object.
pub fn require_f64(params: &Value, key: &str) -> Result<f64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract an optional bool parameter from a JSON object.
pub fn optional_bool(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    params.get(key).map_or(Ok(None), |v| {
        v.as_bool()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a boolean"),
                param: key.to_string(),
            })
    })
}
