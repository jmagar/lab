use serde_json::Value;

use crate::dispatch::error::ToolError;

pub const ACTIONS: &[lab_apis::core::action::ActionSpec] = crate::dispatch::logs::ACTIONS;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::logs::dispatch(action, params).await
}
