//! MCP adapter for the `tailscale` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::tailscale::dispatch(action, params).await
}
