//! MCP adapter for the `prowlarr` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::prowlarr::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::prowlarr::dispatch(action, params).await
}
