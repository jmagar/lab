use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::json;

use crate::api::{ToolError, state::AppState};
use crate::config::DeviceRole;

pub async fn list_devices(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let store = require_master_store(&state)?;
    let devices = store.list_devices().await;
    Ok(Json(serde_json::Value::Array(
        devices
            .into_iter()
            .map(|snapshot| {
                json!({
                    "device_id": snapshot.device_id,
                    "connected": snapshot.connected,
                    "role": snapshot.role,
                    "log_count": snapshot.logs.len(),
                    "discovered_config_count": snapshot
                        .metadata
                        .as_ref()
                        .map(|metadata| metadata.discovered_configs.len())
                        .unwrap_or(0),
                })
            })
            .collect(),
    )))
}

pub async fn get_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let store = require_master_store(&state)?;
    let snapshot = store
        .device(&device_id)
        .await
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: format!("unknown device `{device_id}`"),
        })?;
    Ok(Json(json!({
        "device_id": snapshot.device_id,
        "connected": snapshot.connected,
        "role": snapshot.role,
        "status": snapshot.status,
        "metadata": snapshot.metadata,
        "log_count": snapshot.logs.len(),
    })))
}

pub(crate) fn require_master_store(
    state: &AppState,
) -> Result<std::sync::Arc<crate::device::store::DeviceFleetStore>, ToolError> {
    if matches!(state.device_role, Some(DeviceRole::NonMaster)) {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: "device fleet queries are only available on the master".to_string(),
        });
    }
    state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))
}
