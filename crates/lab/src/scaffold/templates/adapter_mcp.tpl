use serde_json::Value;

use crate::dispatch::error::ToolError;

pub use crate::dispatch::{{service}}::ACTIONS;

/// Forward an MCP action to the scaffolded dispatch layer.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::{{service}}::dispatch(action, params).await
}
