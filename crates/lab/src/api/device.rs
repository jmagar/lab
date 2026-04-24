use axum::{Json, Router, routing::post};
use serde::Serialize;

use super::state::AppState;
use crate::api::ToolError;

pub mod fleet;
pub mod hello;
pub mod enrollments;
pub mod logs;
pub mod metadata;
pub mod oauth;
pub mod status;
pub mod syslog;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceAck {
    pub ok: bool,
}

/// Unauthenticated node routes — mounted outside the bearer-auth middleware.
/// `/v1/nodes/hello` is self-registration and must not require a pre-shared token.
pub fn public_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/hello", post(hello::handle))
        .with_state(state)
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/status", post(status::handle))
        .route("/metadata", post(metadata::handle))
        .route("/enrollments", axum::routing::get(enrollments::list))
        .route(
            "/enrollments/{node_id}/approve",
            post(enrollments::approve),
        )
        .route("/enrollments/{node_id}/deny", post(enrollments::deny))
        .route("", axum::routing::get(fleet::list_devices))
        .route("/{node_id}", axum::routing::get(fleet::get_device))
        .route("/logs/search", post(logs::search))
        .route("/oauth/relay/start", post(oauth::handle_start))
        .route("/syslog/batch", post(syslog::handle_batch))
        .with_state(state)
}

fn ok() -> Json<DeviceAck> {
    Json(DeviceAck { ok: true })
}

pub(crate) fn normalize_node_id_value(node_id: &str, param: &str) -> Result<String, ToolError> {
    let trimmed = node_id.trim();
    if trimmed.is_empty() || trimmed.len() > 256 {
        return Err(ToolError::InvalidParam {
            message: "node_id must be 1-256 non-whitespace characters".to_string(),
            param: param.to_string(),
        });
    }

    Ok(trimmed.to_string())
}

pub(crate) fn normalize_device_id_value(
    device_id: &str,
    param: &str,
) -> Result<String, ToolError> {
    normalize_node_id_value(device_id, param)
}
