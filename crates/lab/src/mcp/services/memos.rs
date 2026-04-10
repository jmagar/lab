//! MCP adapter for the `memos` tool — forwards to dispatch layer.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::memos::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::memos::dispatch(action, params).await
}
