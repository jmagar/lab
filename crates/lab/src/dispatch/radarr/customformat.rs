//! Custom format dispatch.

use lab_apis::core::action::ActionSpec;
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "customformat.list",
    description: "List all custom formats defined in Radarr",
    destructive: false,
    returns: "CustomFormat[]",
    params: &[],
}];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    _action: &str,
    _params: Value,
) -> Result<Value, ToolError> {
    let result = client.customformat_list().await?;
    Ok(result)
}
