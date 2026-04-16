use axum::{Json, extract::State};
use serde_json::Value;

use crate::api::{ToolError, device::DeviceAck, state::AppState};

pub async fn handle(
    State(state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<DeviceAck>, ToolError> {
    let _store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    Ok(super::ok())
}
