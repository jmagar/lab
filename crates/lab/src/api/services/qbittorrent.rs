//! HTTP route group for the `qbittorrent` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::qbittorrent::{ACTIONS, not_configured_error};

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, crate::dispatch::error::ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());

    #[cfg(feature = "qbittorrent")]
    {
        let client = state
            .clients
            .qbittorrent
            .as_deref()
            .ok_or_else(not_configured_error)?;

        return handle_action(
            "qbittorrent",
            "api",
            request_id,
            req,
            ACTIONS,
            |action, params| async move {
                crate::dispatch::qbittorrent::dispatch_with_client(client, &action, params).await
            },
        )
        .await;
    }

    #[allow(unreachable_code)]
    handle_action(
        "qbittorrent",
        "api",
        request_id,
        req,
        ACTIONS,
        |action, params| async move {
            crate::dispatch::qbittorrent::dispatch(&action, params).await
        },
    )
    .await
}
