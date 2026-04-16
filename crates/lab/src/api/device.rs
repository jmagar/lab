use axum::{Json, Router, routing::post};
use serde::Serialize;

use super::state::AppState;
use crate::api::ToolError;

pub mod fleet;
pub mod hello;
pub mod logs;
pub mod metadata;
pub mod oauth;
pub mod status;
pub mod syslog;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceAck {
    pub ok: bool,
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/hello", post(hello::handle))
        .route("/status", post(status::handle))
        .route("/metadata", post(metadata::handle))
        .route("/devices", axum::routing::get(fleet::list_devices))
        .route(
            "/devices/{device_id}",
            axum::routing::get(fleet::get_device),
        )
        .route("/logs/search", post(logs::search))
        .route("/oauth/relay/start", post(oauth::handle_start))
        .route("/syslog/batch", post(syslog::handle_batch))
        .with_state(state)
}

fn ok() -> Json<DeviceAck> {
    Json(DeviceAck { ok: true })
}

pub(crate) fn normalize_device_id_value(device_id: &str, param: &str) -> Result<String, ToolError> {
    let trimmed = device_id.trim();
    if trimmed.is_empty() || trimmed.len() > 256 {
        return Err(ToolError::InvalidParam {
            message: "device_id must be 1-256 non-whitespace characters".to_string(),
            param: param.to_string(),
        });
    }

    Ok(trimmed.to_string())
}
