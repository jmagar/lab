//! HTTP route group for the `setup` Bootstrap orchestrator.
//!
//! Mounted at `/v1/setup` behind the host-validation Layer (Chunk E):
//! requests with a non-loopback Host header are rejected with 421 before
//! reaching the dispatcher.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::setup::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    handle_action(
        "setup",
        "api",
        request_id,
        req,
        ACTIONS,
        |action, params| async move { crate::dispatch::setup::dispatch(&action, params).await },
    )
    .await
}
