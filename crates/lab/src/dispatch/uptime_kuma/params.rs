use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_u32, require_i64};

pub fn monitor_id(params: &Value, key: &str) -> Result<i64, ToolError> {
    require_i64(params, key)
}

pub fn config(params: &Value) -> Result<Value, ToolError> {
    let value = params
        .get("config")
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `config`".into(),
            param: "config".into(),
        })?;
    if value.as_object().is_none() {
        return Err(ToolError::InvalidParam {
            message: "parameter `config` must be an object".into(),
            param: "config".into(),
        });
    }
    Ok(value.clone())
}

pub fn optional_monitor_id(params: &Value) -> Result<Option<i64>, ToolError> {
    match params.get("monitor_id") {
        None => Ok(None),
        Some(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `monitor_id` must be an integer".into(),
                param: "monitor_id".into(),
            }),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn config_requires_object() {
        assert!(config(&json!({ "config": { "type": "http" } })).is_ok());
        assert!(matches!(
            config(&json!({ "config": "nope" })),
            Err(ToolError::InvalidParam { .. })
        ));
    }

    #[test]
    fn hours_clamps_window() {
        assert_eq!(hours(&json!({})).unwrap(), 24);
        assert_eq!(hours(&json!({ "hours": 168 })).unwrap(), 168);
        assert!(matches!(
            hours(&json!({ "hours": 169 })),
            Err(ToolError::InvalidParam { .. })
        ));
    }
}

pub fn hours(params: &Value) -> Result<u32, ToolError> {
    let hours = optional_u32(params, "hours")?.unwrap_or(24);
    if !(1..=168).contains(&hours) {
        return Err(ToolError::InvalidParam {
            message: "parameter `hours` must be between 1 and 168".into(),
            param: "hours".into(),
        });
    }
    Ok(hours)
}
