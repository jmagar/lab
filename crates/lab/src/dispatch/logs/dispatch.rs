// Stub — replaced by Task 10.
use serde_json::Value;

use super::types::LogSystem;
use crate::dispatch::error::ToolError;

pub async fn dispatch(_action: &str, _params: Value) -> Result<Value, ToolError> {
    unimplemented!("dispatch stub (Task 10)")
}

pub async fn dispatch_with_system(
    _system: &LogSystem,
    _action: &str,
    _params: Value,
) -> Result<Value, ToolError> {
    unimplemented!("dispatch_with_system stub (Task 10)")
}
