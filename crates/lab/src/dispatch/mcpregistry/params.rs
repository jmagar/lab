use lab_apis::mcpregistry::types::ListServersParams;
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Extract `server.list` params from the dispatch params object.
///
/// Maps:
/// - `query`  → `ListServersParams::search`
/// - `limit`  → `ListServersParams::limit`
/// - `cursor` → `ListServersParams::cursor`
pub fn list_servers_params(params: &Value) -> Result<ListServersParams, ToolError> {
    let search = params["query"].as_str().map(str::to_string);
    let limit = params["limit"].as_u64().map(|v| v as u32);
    let cursor = params["cursor"].as_str().map(str::to_string);
    Ok(ListServersParams {
        search,
        cursor,
        limit,
        version: None,
        updated_since: None,
    })
}

/// Extract a required `name` string param.
pub fn require_name(params: &Value) -> Result<String, ToolError> {
    match params["name"].as_str() {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        Some(_) | None => Err(ToolError::MissingParam {
            message: "missing required parameter `name`".to_string(),
            param: "name".to_string(),
        }),
    }
}
