use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, state::AppState};
use crate::config::DeviceRole;

#[derive(Debug, Deserialize)]
pub struct SearchLogsRequest {
    pub device_id: String,
    pub query: String,
}

pub async fn search(
    State(state): State<AppState>,
    Json(payload): Json<SearchLogsRequest>,
) -> Result<Json<Vec<crate::device::log_event::DeviceLogEvent>>, ToolError> {
    if matches!(state.device_role, Some(DeviceRole::NonMaster)) {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: "device fleet logs are only available on the master".to_string(),
        });
    }

    let store = state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))?;
    let needle = payload.query.to_ascii_lowercase();
    let events = store
        .logs_for_device(&payload.device_id)
        .await
        .into_iter()
        .filter(|event| event.message.to_ascii_lowercase().contains(&needle))
        .collect();
    Ok(Json(events))
}
