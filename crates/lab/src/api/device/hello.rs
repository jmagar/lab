use axum::{Json, extract::State};

use crate::api::{ToolError, device::DeviceAck, state::AppState};
use crate::device::checkin::DeviceHello;

pub async fn handle(
    State(state): State<AppState>,
    Json(payload): Json<DeviceHello>,
) -> Result<Json<DeviceAck>, ToolError> {
    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    store.record_hello(payload).await;
    Ok(super::ok())
}
