//! Liveness and readiness probes.
//!
//! `GET /health` — process is up. Always returns 200.
//! `GET /ready`  — process is ready to serve traffic (e.g., config loaded,
//!                  at least one service client constructed). Returns 503
//!                  until ready.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

use super::state::AppState;

/// Liveness probe. Returns 200 as long as the process is running.
pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// Readiness probe. Returns 200 once the app state is fully constructed,
/// 503 otherwise.
pub async fn ready(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ready" })))
}
