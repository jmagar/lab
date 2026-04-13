//! HTTP route group for the `gotify` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::gotify::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.gotify.clone();
    handle_action(
        "gotify",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::gotify::not_configured_error());
            };
            crate::dispatch::gotify::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
