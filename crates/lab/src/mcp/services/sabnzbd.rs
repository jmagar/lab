//! MCP adapter for the `SABnzbd` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/sabnzbd.rs`.
//! MCP wiring (catalog + dispatch) is done directly in `mcp/registry.rs`.

use serde_json::Value;

use crate::dispatch::error::ToolError;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::sabnzbd::dispatch(action, params).await
}
