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
    /// Process role: `"master"` or `"node"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// OS process ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Seconds since the server started accepting requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_s: Option<u64>,
}

/// Liveness probe. Returns 200 as long as the process is running.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let uptime_s = state.server_start.elapsed().as_secs();
    let mode = if state.is_master() { "master" } else { "node" };
    Json(HealthResponse {
        status: "ok".to_string(),
        mode: Some(mode.to_string()),
        pid: Some(std::process::id()),
        uptime_s: Some(uptime_s),
    })
}

/// Readiness probe. Returns 200 once the app state is fully constructed,
/// 503 otherwise.
pub async fn ready(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ready".to_string(),
            mode: None,
            pid: None,
            uptime_s: None,
        }),
    )
}
