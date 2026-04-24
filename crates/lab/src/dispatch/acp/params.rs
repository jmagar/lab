//! Param coercion helpers for ACP dispatch actions.

use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Extract a required string param. Returns `MissingParam` if absent or null.
pub fn require_str<'a>(params: &'a Value, name: &str) -> Result<&'a str, ToolError> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("required param `{name}` is missing or empty"),
            param: name.to_string(),
        })
}

/// Extract an optional string param. Returns `None` if absent, null, or empty.
pub fn opt_str<'a>(params: &'a Value, name: &str) -> Option<&'a str> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Extract an optional u64 param.
pub fn opt_u64(params: &Value, name: &str) -> Result<Option<u64>, ToolError> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("`{name}` must be a non-negative integer"),
                param: name.to_string(),
            }),
    }
}
