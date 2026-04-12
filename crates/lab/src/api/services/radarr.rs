//! API route group for the `radarr` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
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
    let client = state.clients.radarr.clone();
    handle_action(
        "radarr",
        "api",
        request_id,
        req,
        crate::dispatch::radarr::actions(),
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: "RADARR_URL or RADARR_API_KEY not configured".into(),
                });
            };
            crate::dispatch::radarr::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
