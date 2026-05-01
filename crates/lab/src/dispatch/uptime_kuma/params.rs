use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_u32, require_i64};

pub fn monitor_id(params: &Value, key: &str) -> Result<i64, ToolError> {
    require_i64(params, key)
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
