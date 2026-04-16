use axum::{Json, Router, routing::post};
use serde::Serialize;

use super::state::AppState;

pub mod hello;
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
        .route("/oauth/relay/start", post(oauth::handle_start))
        .route("/syslog/batch", post(syslog::handle_batch))
        .with_state(state)
}

fn ok() -> Json<DeviceAck> {
    Json(DeviceAck { ok: true })
}
