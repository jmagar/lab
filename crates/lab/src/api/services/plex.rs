//! HTTP route group for the `plex` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::context::DispatchContext;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::dispatch::error::ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    handle_action(
        "plex",
        DispatchContext { surface: "api", instance: None },
        request_id,
        req,
        crate::dispatch::plex::ACTIONS,
        |action, params| async move {
            crate::dispatch::plex::dispatch(&action, params).await
        },
        Some(&headers),
    )
    .await
}
