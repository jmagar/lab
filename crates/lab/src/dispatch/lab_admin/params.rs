use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Extract the required `services` array from params.
///
/// Returns a `Vec<String>` of service names, or a structured error if the
/// param is absent or contains non-string elements.
pub fn parse_services(params: &Value) -> Result<Vec<String>, ToolError> {
    params
        .get("services")
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `services`".into(),
            param: "services".into(),
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| ToolError::InvalidParam {
                    message: "parameter `services` must be an array of strings".into(),
                    param: "services".into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()
}
