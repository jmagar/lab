use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

pub fn title(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "title")
}

pub fn notebook_id(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "notebook_id")
}

pub fn url(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "url")
}
