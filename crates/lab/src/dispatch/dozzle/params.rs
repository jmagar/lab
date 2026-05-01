//! Parameter helpers for the `dozzle` service.

use lab_apis::dozzle::types::{LogFetchRequest, ReadLimits};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_u32, require_str};

pub fn limits_from_params(params: &Value) -> Result<ReadLimits, ToolError> {
    let defaults = ReadLimits::default();
    Ok(ReadLimits {
        max_events: optional_usize(params, "max_events")?.unwrap_or(defaults.max_events),
        max_lines: optional_usize(params, "max_lines")?.unwrap_or(defaults.max_lines),
        max_bytes: optional_usize(params, "max_bytes")?.unwrap_or(defaults.max_bytes),
        timeout_ms: optional_u64(params, "timeout_ms")?.unwrap_or(defaults.timeout_ms),
    }
    .capped())
}

pub fn log_fetch_request_from_params(params: &Value) -> Result<LogFetchRequest, ToolError> {
    let host = require_str(params, "host")?.to_string();
    let container_id = require_str(params, "container_id")?.to_string();
    let stdout = optional_bool(params, "stdout")?;
    let stderr = optional_bool(params, "stderr")?;
    let (stdout, stderr) = match (stdout, stderr) {
        (None, None) => (true, true),
        (Some(stdout), None) => (stdout, false),
        (None, Some(stderr)) => (false, stderr),
        (Some(stdout), Some(stderr)) => (stdout, stderr),
    };
    if !stdout && !stderr {
        return Err(ToolError::InvalidParam {
            message: "at least one of `stdout` or `stderr` must be true".into(),
            param: "stdout".into(),
        });
    }
    Ok(LogFetchRequest {
        host,
        container_id,
        stdout,
        stderr,
        limits: limits_from_params(params)?,
    })
}

fn optional_usize(params: &Value, key: &str) -> Result<Option<usize>, ToolError> {
    optional_u32(params, key).map(|value| value.map(|n| n as usize))
}

fn optional_u64(params: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    params.get(key).map_or(Ok(None), |value| {
        value
            .as_u64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a non-negative integer"),
                param: key.to_string(),
            })
    })
}

fn optional_bool(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    params.get(key).map_or(Ok(None), |value| {
        value
            .as_bool()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a boolean"),
                param: key.to_string(),
            })
    })
}
