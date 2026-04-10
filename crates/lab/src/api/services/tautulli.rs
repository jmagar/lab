//! HTTP route group for the `tautulli` service.

use axum::{Json, Router, extract::State, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::context::DispatchContext;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(_state): State<AppState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::dispatch::error::ToolError> {
    handle_action(
        "tautulli",
        DispatchContext { surface: "api", instance: None },
        None,
        req,
        crate::dispatch::tautulli::ACTIONS,
        |action, params| async move {
            crate::dispatch::tautulli::dispatch(&action, params).await
        },
    )
    .await
}
