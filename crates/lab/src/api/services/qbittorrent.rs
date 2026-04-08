//! HTTP route group for the `qbittorrent` service.

use axum::{Json, Router, extract::State, routing::post};
use serde_json::Value;

use crate::api::{ActionRequest, state::AppState};

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::mcp::envelope::ToolError> {
    let start = std::time::Instant::now();
    let action = req.action.clone();
    let result = crate::mcp::services::qbittorrent::dispatch(&req.action, req.params)
        .await
        .map_err(|e| crate::mcp::envelope::ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: e.to_string(),
        });
    let elapsed_ms = start.elapsed().as_millis();
    match &result {
        Ok(_) => tracing::info!(service = "qbittorrent", action, elapsed_ms, "dispatch ok"),
        Err(e) => tracing::warn!(
            service = "qbittorrent",
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }
    result.map(Json)
}
