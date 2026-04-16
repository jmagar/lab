use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::log_event::DeviceLogEvent;

#[derive(Debug, Deserialize)]
pub struct DeviceSyslogBatch {
    pub device_id: String,
    pub events: Vec<DeviceLogEvent>,
}

pub async fn handle_batch(
    State(state): State<AppState>,
    Json(_payload): Json<DeviceSyslogBatch>,
) -> Result<Json<DeviceAck>, ToolError> {
    let _store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    Ok(super::ok())
}
