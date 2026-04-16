use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, state::AppState};

use super::{fleet::require_master_store, validate_device_id_value};

#[derive(Debug, Deserialize)]
pub struct SearchLogsRequest {
    pub device_id: String,
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

pub async fn search(
    State(state): State<AppState>,
    Json(payload): Json<SearchLogsRequest>,
) -> Result<Json<Vec<crate::device::log_event::DeviceLogEvent>>, ToolError> {
    validate_device_id_value(&payload.device_id, "device_id")?;
    let store = require_master_store(&state)?;
    let needle = payload.query.to_ascii_lowercase();
    let offset = payload.offset.unwrap_or(0);
    let limit = payload.limit.unwrap_or(200);
    let events = store
        .search_logs_for_device(&payload.device_id, &needle, offset, limit)
        .await
        .into_iter()
        .collect();
    Ok(Json(events))
}
