//! HTTP route group for the `sonarr` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::dispatch::error::ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    handle_action(
        "sonarr",
        "api",
        request_id,
        req,
        crate::dispatch::sonarr::ACTIONS,
        |action, params| async move { crate::dispatch::sonarr::dispatch(&action, params).await },
    )
    .await
}
