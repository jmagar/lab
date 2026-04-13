//! HTTP route group for the `unraid` service.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
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
    let client = state.clients.unraid.clone();
    handle_action(
        "unraid",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let instance = crate::dispatch::helpers::optional_str(&params, "instance")?
                .map(str::to_owned);
            let mut params_clean = params;
            if let Value::Object(ref mut map) = params_clean {
                map.remove("instance");
            }
            match instance {
                Some(label) => {
                    let c = crate::dispatch::unraid::client_from_instance(Some(&label))?;
                    crate::dispatch::unraid::dispatch_with_client(&c, &action, params_clean).await
                }
                None => {
                    let Some(ref c) = client else {
                        return Err(crate::dispatch::unraid::not_configured_error());
                    };
                    crate::dispatch::unraid::dispatch_with_client(c, &action, params_clean).await
                }
            }
        },
    )
    .await
}
