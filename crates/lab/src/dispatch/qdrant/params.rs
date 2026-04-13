use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Extract and validate a collection `name` parameter.
///
/// Returns `MissingParam` when absent and `InvalidParam` when empty.
pub fn collection_name_from_params(params: &Value) -> Result<&str, ToolError> {
    let name = require_str(params, "name")?.trim();
    if name.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `name` must not be empty".into(),
            param: "name".into(),
        });
    }
    Ok(name)
}
