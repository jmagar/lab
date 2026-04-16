use std::time::Instant;

use axum::{Json, extract::State, http::HeaderMap};
use serde::Deserialize;

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::log_event::DeviceLogEvent;

use super::normalize_device_id_value;

#[derive(Debug, Deserialize)]
pub struct DeviceSyslogBatch {
    pub device_id: String,
    pub events: Vec<DeviceLogEvent>,
}

pub async fn handle_batch(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(mut payload): Json<DeviceSyslogBatch>,
) -> Result<Json<DeviceAck>, ToolError> {
    let start = Instant::now();
    let device_id = normalize_device_id_value(&payload.device_id, "device_id")?;
    for (index, event) in payload.events.iter_mut().enumerate() {
        let event_id =
            normalize_device_id_value(&event.device_id, &format!("events[{index}].device_id"))?;
        if event_id != device_id {
            return Err(ToolError::InvalidParam {
                message: format!(
                    "events[{index}].device_id must match batch device_id `{device_id}`"
                ),
                param: format!("events[{index}].device_id"),
            });
        }
        event.device_id = device_id.clone();
    }

    let event_count = payload.events.len();
    let request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok());
    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    store.record_logs(&device_id, payload.events).await;
    tracing::info!(
        surface = "api",
        service = "device",
        action = "syslog.batch",
        request_id,
        device_id = %device_id,
        event_count,
        elapsed_ms = start.elapsed().as_millis(),
        "device syslog batch recorded"
    );
    Ok(super::ok())
}
