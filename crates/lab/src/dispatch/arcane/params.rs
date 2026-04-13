use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Extract `env_id` from params.
pub fn env_id_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "env_id").map(str::to_string)
}

/// Extract `container_id` from params.
pub fn container_id_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "container_id").map(str::to_string)
}

/// Extract `id` from params (used for single-resource lookups).
pub fn id_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "id").map(str::to_string)
}
