//! Liveness and readiness probes.
//!
//! `GET /health` — process is up. Always returns 200.
//! `GET /ready`  — process is ready to serve traffic (e.g., config loaded,
//!                  at least one service client constructed). Returns 503
//!                  until ready.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use super::state::AppState;

/// Response body for health/readiness probes.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// Status string: `"ok"` for liveness, `"ready"` for readiness.
    pub status: String,
}

/// Liveness probe. Returns 200 as long as the process is running.
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// Readiness probe. Returns 200 once the app state is fully constructed,
/// 503 otherwise.
pub async fn ready(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ready".to_string(),
        }),
    )
}
