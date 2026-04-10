//! Parameter extraction helpers for the `unraid` dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Extract the required `id` parameter from a params object.
pub fn require_id(params: &Value) -> Result<String, ToolError> {
    require_str(params, "id").map(ToOwned::to_owned)
}
