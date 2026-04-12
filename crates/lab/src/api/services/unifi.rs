//! HTTP route group for the `unifi` service.
//!
use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::optional_str;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id_owned = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let request_id = request_id_owned.as_deref();
    handle_action(
        "unifi",
        "api",
        request_id,
        req,
        crate::dispatch::unifi::actions(),
        |action, params| async move { crate::dispatch::unifi::dispatch(&action, params).await },
    )
    .await
}
