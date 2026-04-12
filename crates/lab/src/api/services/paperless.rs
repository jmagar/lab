//! HTTP route group for the `paperless` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::paperless::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.paperless.clone();
    handle_action(
        "paperless",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: "PAPERLESS_URL or PAPERLESS_TOKEN not configured".into(),
                });
            };
            crate::dispatch::paperless::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
