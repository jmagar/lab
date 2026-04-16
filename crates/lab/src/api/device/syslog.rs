use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::log_event::DeviceLogEvent;

use super::validate_device_id_value;

#[derive(Debug, Deserialize)]
pub struct DeviceSyslogBatch {
    pub device_id: String,
    pub events: Vec<DeviceLogEvent>,
}

pub async fn handle_batch(
    State(state): State<AppState>,
    Json(payload): Json<DeviceSyslogBatch>,
) -> Result<Json<DeviceAck>, ToolError> {
    validate_device_id_value(&payload.device_id, "device_id")?;
    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    store.record_logs(&payload.device_id, payload.events).await;
    Ok(super::ok())
}
