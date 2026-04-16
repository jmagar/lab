use axum::{Json, extract::State};

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::checkin::DeviceHello;

pub async fn handle(
    State(state): State<AppState>,
    Json(mut payload): Json<DeviceHello>,
) -> Result<Json<DeviceAck>, ToolError> {
    payload.device_id = super::normalize_device_id_value(&payload.device_id, "device_id")?;
    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    store.record_hello(payload).await;
    Ok(super::ok())
}
