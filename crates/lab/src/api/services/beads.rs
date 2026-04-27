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
            // help/schema bypass the client; dispatch_with_client also handles them
            // but dispatch() avoids needing a configured client.
            if matches!(action.as_str(), "help" | "schema") {
                return crate::dispatch::beads::dispatch(&action, params).await;
            }
            let Some(client) = client.as_ref() else {
                return Err(ToolError::Sdk {
                    sdk_kind: "beads_unavailable".into(),
                    message: "beads database is not reachable".into(),
                });
            };
            crate::dispatch::beads::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
