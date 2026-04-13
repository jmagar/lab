use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Extract `device_id` from params.
pub fn device_id_from_params(params: &Value) -> Result<String, ToolError> {
    Ok(require_str(params, "device_id")?.to_string())
}

/// Extract `key_id` from params.
pub fn key_id_from_params(params: &Value) -> Result<String, ToolError> {
    Ok(require_str(params, "key_id")?.to_string())
}

/// Extract `authorized` bool from params.
pub fn authorized_from_params(params: &Value) -> Result<bool, ToolError> {
    match params.get("authorized") {
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(ToolError::InvalidParam {
            message: format!("'authorized' must be a boolean, got: {other}"),
            param: "authorized".to_string(),
        }),
        None => Err(ToolError::MissingParam {
            message: "missing required parameter 'authorized'".to_string(),
            param: "authorized".to_string(),
        }),
    }
}
