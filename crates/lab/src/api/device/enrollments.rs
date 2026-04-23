use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::api::{ToolError, state::AppState};

use super::normalize_device_id_value;

#[derive(Debug, Deserialize, Default)]
pub struct ApproveEnrollmentRequest {
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DenyEnrollmentRequest {
    pub reason: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ToolError> {
    let store = super::fleet::require_enrollment_store(&state)?;
    let snapshot = store
        .list()
        .await
        .map_err(|error| ToolError::internal_message(format!("list enrollments: {error}")))?;
    Ok(Json(serde_json::to_value(snapshot).map_err(|error| {
        ToolError::internal_message(format!("serialize enrollments: {error}"))
    })?))
}

pub async fn approve(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(payload): Json<ApproveEnrollmentRequest>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let device_id = normalize_device_id_value(&device_id, "device_id")?;
    let store = super::fleet::require_enrollment_store(&state)?;
    let approved = store
        .approve(&device_id, payload.note)
        .await
        .map_err(|error| ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: error.to_string(),
        })?;
    Ok(Json(serde_json::to_value(approved).map_err(|error| {
        ToolError::internal_message(format!("serialize approved enrollment: {error}"))
    })?))
}

pub async fn deny(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(payload): Json<DenyEnrollmentRequest>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let device_id = normalize_device_id_value(&device_id, "device_id")?;
    let store = super::fleet::require_enrollment_store(&state)?;
    let denied = store
        .deny(&device_id, payload.reason)
        .await
        .map_err(|error| ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: error.to_string(),
        })?;
    Ok(Json(serde_json::to_value(denied).map_err(|error| {
        ToolError::internal_message(format!("serialize denied enrollment: {error}"))
    })?))
}
