//! HTTP route group for the `bytestash` service.

use axum::{Json, Router, extract::State, routing::post};
use serde_json::Value;

use crate::api::{ActionRequest, state::AppState};
use crate::api::services::helpers::handle_action;
use crate::services::bytestash::ACTIONS;
use crate::services::context::DispatchContext;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::mcp::envelope::ToolError> {
    handle_action("bytestash", DispatchContext { surface: "api", instance: None }, req, ACTIONS, |action, params| async move {
        crate::services::bytestash::dispatch(&action, params).await
    })
    .await
}
