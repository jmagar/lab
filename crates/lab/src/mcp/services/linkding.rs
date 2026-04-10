//! MCP adapter for the `linkding` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::linkding::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::linkding::dispatch(action, params).await
}
