use axum::{Json, extract::State};

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::checkin::DeviceStatus;

pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<DeviceStatus>,
) -> Result<Json<DeviceAck>, ToolError> {
    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    store.record_status(payload).await;
    Ok(super::ok())
}
