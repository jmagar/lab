//! Param coercion helpers for ACP dispatch actions.
//!
//! ACP-specific semantic: empty strings are treated as missing/absent. The
//! shared `helpers::require_str` does not filter empties, and
//! `helpers::optional_str` rejects empties as `InvalidParam` — neither matches
//! ACP's contract. These wrappers preserve the empty-as-missing behavior on
//! top of the shared helpers.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers;

/// Extract a required string param. Returns `MissingParam` if absent, null, or empty.
pub fn require_str<'a>(params: &'a Value, name: &str) -> Result<&'a str, ToolError> {
    let value = helpers::require_str(params, name)?;
    if value.is_empty() {
        return Err(ToolError::MissingParam {
            message: format!("required param `{name}` is missing or empty"),
            param: name.to_string(),
        });
    }
    Ok(value)
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
        Some(v) => v.as_u64().map(Some).ok_or_else(|| ToolError::InvalidParam {
            message: format!("`{name}` must be a non-negative integer"),
            param: name.to_string(),
        }),
    }
}
