//! HTTP route group for the `marketplace` service.

use std::future::Future;
use std::pin::Pin;

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::marketplace::NodeRpcPort;
use crate::dispatch::node::send::send_rpc_to_node;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

/// Concrete `NodeRpcPort` impl that sends JSON-RPC requests over the node
/// WebSocket and awaits the response via the master-side pending map.
///
/// Built here (not in `dispatch/`) so the dispatch layer stays free of any
/// master-transport concerns. The dispatch layer only sees the trait.
pub(crate) struct WsNodeRpcPort;

impl NodeRpcPort for WsNodeRpcPort {
    fn send_rpc(
        &self,
        node_id: &str,
        method: &str,
        params: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send + '_>> {
        let node_id = node_id.to_string();
        let method = method.to_string();
        Box::pin(async move { send_rpc_to_node(&node_id, &method, params).await })
    }
}

async fn handle(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    handle_action(
        "marketplace",
        "api",
        request_id,
        req,
        crate::dispatch::marketplace::ACTIONS,
        |action, params| async move {
            // Route all marketplace actions through `dispatch_with_port` with
            // the real fleet-backed port, so cherry_pick can actually reach
            // connected nodes. Non-cherry-pick actions ignore the port.
            crate::dispatch::marketplace::dispatch_with_port(
                &action,
                params,
                &WsNodeRpcPort,
            )
            .await
        },
    )
    .await
}
