//! MCP dispatch stub for the `overseerr` tool.
//!
//! Replace this stub with a real implementation when the service client
//! is ready. See `radarr.rs` for the reference pattern.

use serde_json::Value;

use crate::mcp::envelope::ToolError;

use lab_apis::core::action::ActionSpec;

/// Action catalog — empty until service is implemented.
pub const ACTIONS: &[ActionSpec] = &[];

/// Dispatch one MCP call against the overseerr tool.
///
/// # Errors
/// Returns `not_implemented` for all actions until the service is wired.
pub async fn dispatch(action: &str, _params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "overseerr",
            "message": "overseerr is not yet implemented",
            "actions": []
        })),
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
