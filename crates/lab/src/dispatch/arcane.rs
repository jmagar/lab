//! Dispatch stub for `arcane`. Replace with real impl when SDK client is ready.

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Action catalog — empty until service is implemented.
pub const ACTIONS: &[ActionSpec] = &[];

/// Dispatch one call against the `arcane` tool.
///
/// # Errors
/// Returns errors for unknown actions until the service is wired.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "arcane",
            "message": "arcane is not yet implemented",
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
