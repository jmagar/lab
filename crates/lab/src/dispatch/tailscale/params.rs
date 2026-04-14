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

/// Extract a required JSON object from a named param field.
pub fn required_object<'a>(params: &'a Value, name: &str) -> Result<&'a Value, ToolError> {
    match params.get(name) {
        Some(v) if v.is_object() => Ok(v),
        Some(other) => Err(ToolError::InvalidParam {
            message: format!("'{name}' must be a JSON object, got: {other}"),
            param: name.to_string(),
        }),
        None => Err(ToolError::MissingParam {
            message: format!("missing required parameter '{name}'"),
            param: name.to_string(),
        }),
    }
}

/// Extract a required array of strings from params.
pub fn required_string_array(params: &Value, name: &str) -> Result<Vec<String>, ToolError> {
    match params.get(name) {
        Some(Value::Array(arr)) => {
            let mut result = Vec::with_capacity(arr.len());
            for item in arr {
                match item.as_str() {
                    Some(s) => result.push(s.to_string()),
                    None => {
                        return Err(ToolError::InvalidParam {
                            message: format!("'{name}' must be an array of strings"),
                            param: name.to_string(),
                        });
                    }
                }
            }
            Ok(result)
        }
        Some(other) => Err(ToolError::InvalidParam {
            message: format!("'{name}' must be an array of strings, got: {other}"),
            param: name.to_string(),
        }),
        None => Err(ToolError::MissingParam {
            message: format!("missing required parameter '{name}'"),
            param: name.to_string(),
        }),
    }
}
