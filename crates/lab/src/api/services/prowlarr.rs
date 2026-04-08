//! HTTP route group for the `prowlarr` service.

use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::Value;

use crate::api::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub params: Value,
}

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::mcp::envelope::ToolError> {
    let start = std::time::Instant::now();
    let action = req.action.clone();
    let result = crate::mcp::services::prowlarr::dispatch(&req.action, req.params)
        .await
        .map_err(|e| crate::mcp::envelope::ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: e.to_string(),
        });
    let elapsed_ms = start.elapsed().as_millis();
    match &result {
        Ok(_) => tracing::info!(service = "prowlarr", action, elapsed_ms, "dispatch ok"),
        Err(e) => tracing::warn!(
            service = "prowlarr",
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }
    result.map(Json)
}
