use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::helpers::{require_str, to_json};

/// Extract a required u64 parameter from a JSON object.
pub fn require_u64(params: &Value, key: &str) -> Result<u64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract an optional u32 parameter, clamping any overflow to `u32::MAX`.
pub fn opt_u32(params: &Value, key: &str) -> Option<u32> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .map(|n| u32::try_from(n).unwrap_or(u32::MAX))
}
