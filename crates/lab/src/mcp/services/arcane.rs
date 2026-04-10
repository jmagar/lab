//! MCP adapter for the `arcane` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::arcane::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::arcane::dispatch(action, params).await
}
