//! Parameter extraction helpers for the `unraid` dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, require_str};

/// Extract the required `id` parameter from a params object.
pub fn require_id(params: &Value) -> Result<String, ToolError> {
    require_str(params, "id").map(ToOwned::to_owned)
}

/// Extract the required `path` parameter from a params object.
pub fn require_path(params: &Value) -> Result<String, ToolError> {
    require_str(params, "path").map(ToOwned::to_owned)
}

/// Extract an optional `lines` u32 from a params object.
pub fn optional_lines(params: &Value) -> Result<Option<u32>, ToolError> {
    match params.get("lines") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "`lines` must be a non-negative integer ≤ 4294967295".to_string(),
                param: "lines".to_string(),
            }),
        Some(other) => Err(ToolError::InvalidParam {
            message: format!("`lines` must be an integer, got {other}"),
            param: "lines".to_string(),
        }),
    }
}

/// Extract an optional `correcting` bool from a params object.
pub fn optional_correcting(params: &Value) -> Result<Option<bool>, ToolError> {
    match params.get("correcting") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(other) => Err(ToolError::InvalidParam {
            message: format!("`correcting` must be a bool, got {other}"),
            param: "correcting".to_string(),
        }),
    }
}

/// Extract the required `title` parameter from a params object.
pub fn require_title(params: &Value) -> Result<String, ToolError> {
    require_str(params, "title").map(ToOwned::to_owned)
}

/// Extract an optional `description` string from a params object.
pub fn optional_description(params: &Value) -> Result<Option<String>, ToolError> {
    Ok(optional_str(params, "description")?.map(ToOwned::to_owned))
}

/// Extract an optional `importance` string from a params object.
pub fn optional_importance(params: &Value) -> Result<Option<String>, ToolError> {
    Ok(optional_str(params, "importance")?.map(ToOwned::to_owned))
}

/// Extract an optional `type` string from a params object.
pub fn optional_type(params: &Value) -> Result<Option<String>, ToolError> {
    Ok(optional_str(params, "type")?.map(ToOwned::to_owned))
}
