//! HTTP route group for the `tei` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::tei::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.tei.clone();
    handle_action(
        "tei",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            // help/schema are handled by the top-level dispatch function;
            // dispatch_with_client does not have arms for them.
            if matches!(action.as_str(), "help" | "schema") {
                return crate::dispatch::tei::dispatch(&action, params).await;
            }
            let Some(client) = client.as_ref() else {
                return Err(ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: "TEI_URL not configured".into(),
                });
            };
            crate::dispatch::tei::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
