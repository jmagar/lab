//! MCP dispatch stub for the `unraid` tool.
//!
//! Replace this stub with a real implementation when the service client
//! is ready. See `radarr.rs` for the reference pattern.

use anyhow::Result;
use serde_json::Value;

use lab_apis::core::action::ActionSpec;

/// Action catalog — empty until service is implemented.
pub const ACTIONS: &[ActionSpec] = &[];

/// Dispatch one MCP call against the unraid tool.
///
/// # Errors
/// Returns `not_implemented` for all actions until the service is wired.
pub async fn dispatch(action: &str, _params: Value) -> Result<Value> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "unraid",
            "message": "unraid is not yet implemented",
            "actions": []
        })),
        _ => anyhow::bail!("unraid is not yet implemented — action: {action}"),
    }
}
