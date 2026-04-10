//! Shared dispatch helpers used across all service modules.

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Serialize any `Serialize` value to `serde_json::Value`.
pub fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Extract a required string parameter from a JSON object.
pub fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract a required integer parameter from a JSON object.
pub fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    match params.get(key) {
        None => Err(ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        }),
        Some(v) => v.as_i64().ok_or_else(|| ToolError::InvalidParam {
            message: format!("parameter `{key}` must be an integer"),
            param: key.to_string(),
        }),
    }
}

/// Extract a required non-negative integer parameter from a JSON object.
#[allow(dead_code)]
pub fn require_u32(params: &Value, key: &str) -> Result<u32, ToolError> {
    match params.get(key) {
        None => Err(ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        }),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a non-negative integer"),
                param: key.to_string(),
            }),
    }
}

/// Clone a JSON object, stripping the given keys.
pub fn object_without(params: &Value, strip: &[&str]) -> Value {
    let mut map = params.as_object().cloned().unwrap_or_default();
    for key in strip {
        map.remove(*key);
    }
    Value::Object(map)
}

/// Build the standard `help` response payload for a service.
///
/// Produces the canonical `{ service, actions: [...] }` shape returned by every
/// service dispatcher when `action == "help"`.
pub fn help_payload(service: &str, actions: &[ActionSpec]) -> Value {
    serde_json::json!({
        "service": service,
        "actions": actions.iter().map(|a| serde_json::json!({
            "name": a.name,
            "description": a.description,
            "destructive": a.destructive,
            "returns": a.returns,
            "params": a.params.iter().map(|p| serde_json::json!({
                "name": p.name,
                "type": p.ty,
                "required": p.required,
                "description": p.description,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// Return the schema for one named action.
///
/// Used to implement the `"schema"` built-in action in every service dispatcher.
/// Returns `ToolError::UnknownAction` if `action_name` is not in `actions`.
pub fn action_schema(actions: &[ActionSpec], action_name: &str) -> Result<Value, ToolError> {
    let spec = actions
        .iter()
        .find(|a| a.name == action_name)
        .ok_or_else(|| ToolError::UnknownAction {
            message: format!("no schema for unknown action `{action_name}`"),
            valid: actions.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        })?;
    Ok(serde_json::json!({
        "action": spec.name,
        "description": spec.description,
        "destructive": spec.destructive,
        "returns": spec.returns,
        "params": spec.params.iter().map(|p| serde_json::json!({
            "name": p.name,
            "type": p.ty,
            "required": p.required,
            "description": p.description,
        })).collect::<Vec<_>>(),
    }))
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
