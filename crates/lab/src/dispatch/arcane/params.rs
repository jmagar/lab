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

/// Extract `project_id` from params.
pub fn project_id_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "project_id").map(str::to_string)
}

/// Extract `volume_name` from params.
pub fn volume_name_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "volume_name").map(str::to_string)
}

/// Extract `image` from params (image reference string for pull operations).
pub fn image_from_params(params: &Value) -> Result<String, ToolError> {
    require_str(params, "image").map(str::to_string)
}
