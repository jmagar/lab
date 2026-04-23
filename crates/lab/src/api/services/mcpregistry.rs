//! HTTP route group for the `mcpregistry` service.

use axum::{
    Json, Router, extract::DefaultBodyLimit, extract::State, http::HeaderMap, routing::post,
};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::mcpregistry::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handle))
        .layer(DefaultBodyLimit::max(1_048_576))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.mcpregistry.clone();
    handle_action(
        "mcpregistry",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            if matches!(action.as_str(), "config" | "server.meta.get" | "server.meta.set" | "server.meta.delete") {
                return crate::dispatch::mcpregistry::dispatch(&action, params).await;
            }
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::mcpregistry::client::not_configured_error());
            };
            crate::dispatch::mcpregistry::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}
