use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{state::AppState, ActionRequest};
use crate::dispatch::error::ToolError;
use crate::dispatch::glances::ACTIONS;

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
    let client = state.clients.glances.clone();
    handle_action(
        "glances",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::glances::not_configured_error());
            };
            crate::dispatch::glances::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
