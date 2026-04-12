//! HTTP route group for the `unraid` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::optional_str;
use crate::dispatch::unraid::ACTIONS;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let instance = optional_str(&req.params, "instance")?;
    if instance.is_some() {
        handle_action(
            "unraid",
            "api",
            request_id,
            req,
            ACTIONS,
            |action, params| async move { crate::dispatch::unraid::dispatch(&action, params).await },
            Some(&headers),
        )
        .await
    } else {
        let client = state.clients.unraid.clone().ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: "UNRAID_URL or UNRAID_API_KEY not configured".into(),
        })?;
        handle_action(
            "unraid",
            "api",
            request_id,
            req,
            ACTIONS,
            move |action, params| async move {
                crate::dispatch::unraid::dispatch_with_client(&client, &action, params).await
            },
            Some(&headers),
        )
        .await
    }
}
