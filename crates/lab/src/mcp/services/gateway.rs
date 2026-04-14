use serde_json::Value;

use crate::dispatch::error::ToolError;

pub const ACTIONS: &[lab_apis::core::action::ActionSpec] = crate::dispatch::gateway::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::gateway::dispatch(action, params).await
}
