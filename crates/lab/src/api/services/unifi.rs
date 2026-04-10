//! HTTP route group for the `unifi` service.
//!
//! TODO(perf): unifi sub-dispatchers each call `require_client()` independently.
//! Thread `state.clients.unifi` through sub-dispatchers to enable full
//! connection-pool reuse. See `dispatch/CLAUDE.md` for the migration pattern.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::context::DispatchContext;
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
    // Fail fast if unifi is not configured — avoids dispatching into sub-modules that
    // would each call require_client() and fail with a less informative error.
    if state.clients.unifi.is_none() {
        return Err(ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: "UNIFI_URL or UNIFI_API_KEY not configured".into(),
        });
    }
    handle_action(
        "unifi",
        DispatchContext {
            surface: "api",
            instance: None,
        },
        request_id,
        req,
        crate::dispatch::unifi::actions(),
        |action, params| async move { crate::dispatch::unifi::dispatch(&action, params).await },
        Some(&headers),
    )
    .await
}
