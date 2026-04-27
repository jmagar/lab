//! HTTP route group for the `beads` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::beads::ACTIONS;
use crate::dispatch::error::ToolError;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.beads.clone();
    handle_action(
        "beads",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            crate::dispatch::beads::dispatch_with_optional_client(
                client.as_deref(),
                &action,
                params,
            )
            .await
        },
    )
    .await
}
