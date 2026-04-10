//! MCP dispatch stub for the `memos` tool.
//!
//! Replace this stub with a real implementation when the service client
//! is ready. See `radarr.rs` for the reference pattern.

use serde_json::Value;

use crate::mcp::envelope::ToolError;

use lab_apis::core::action::ActionSpec;

/// Action catalog — empty until service is implemented.
pub const ACTIONS: &[ActionSpec] = &[];

/// Dispatch one MCP call against the memos tool.
///
/// # Errors
/// Returns `not_implemented` for all actions until the service is wired.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "memos",
            "message": "memos is not yet implemented",
            "actions": []
        })),
        "schema" => {
            let a = crate::dispatch::helpers::require_str(&params, "action")?;
            crate::dispatch::helpers::action_schema(ACTIONS, a)
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
