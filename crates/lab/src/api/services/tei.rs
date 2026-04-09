//! HTTP route group for the `tei` service.

use axum::{Json, Router, extract::State, routing::post};
use serde_json::Value;

use crate::api::{ActionRequest, state::AppState};
use crate::api::services::helpers::handle_action;
use crate::services::context::DispatchContext;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::services::error::ToolError> {
    handle_action("tei", DispatchContext { surface: "api", instance: None }, req, crate::mcp::services::tei::ACTIONS, |action, params| async move {
        crate::mcp::services::tei::dispatch(&action, params).await
    })
    .await
}
