//! MCP adapter for the `doctor` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::doctor::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::doctor::dispatch(action, params).await
}
