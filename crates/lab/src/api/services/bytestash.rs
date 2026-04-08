//! HTTP route group stub for the `bytestash` service.

use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::Value;

use crate::api::{error::{ApiError, ApiResult}, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub params: Value,
}

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> ApiResult<Json<Value>> {
    crate::mcp::services::bytestash::dispatch(&req.action, req.params)
        .await
        .map(Json)
        .map_err(|e| ApiError::Internal(e.to_string()))
}
