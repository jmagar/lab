use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::scrutiny::ACTIONS;

/// Build the route group for the scaffolded service.
pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.scrutiny.clone();
    handle_action(
        "scrutiny",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::scrutiny::not_configured_error());
            };
            crate::dispatch::scrutiny::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
