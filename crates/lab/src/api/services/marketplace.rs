//! HTTP route group for the `marketplace` service.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream;
use serde::Deserialize;
use serde_json::Value;
use tracing::{info, warn};

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;
use crate::dispatch::marketplace::NodeRpcPort;
use crate::dispatch::node::send::{send_rpc_to_node, subscribe_progress};

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handle))
        .route("/cherry-pick/progress", get(cherry_pick_progress))
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

// ---------------------------------------------------------------------------
// Cherry-pick SSE progress endpoint (lab-zxx5.16)
// ---------------------------------------------------------------------------
// GET /v1/marketplace/cherry-pick/progress?rpc_id=<uuid>
//
// Subscribes to the per-rpc_id progress broadcast channel and forwards each
// `install/progress` frame as an SSE `data: {json}\n\n` event. The stream
// closes when the broadcast channel is dropped — which happens when the
// correlated RPC response arrives (see `resolve_pending_rpc`).
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub(crate) struct CherryPickProgressQuery {
    rpc_id: String,
}

async fn cherry_pick_progress(
    State(_state): State<AppState>,
    Query(query): Query<CherryPickProgressQuery>,
) -> Result<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>, ToolError> {
    let rpc_id = query.rpc_id.trim();
    if rpc_id.is_empty() {
        return Err(ToolError::MissingParam {
            param: "rpc_id".into(),
            message: "`rpc_id` query parameter is required".into(),
        });
    }
    // Validate rpc_id shape — cherry_pick generates UUIDv4 strings, and we
    // reject anything else to keep a tight surface for this endpoint.
    if uuid::Uuid::parse_str(rpc_id).is_err() {
        return Err(ToolError::InvalidParam {
            param: "rpc_id".into(),
            message: "`rpc_id` must be a UUID".into(),
        });
    }

    let receiver = subscribe_progress(rpc_id);
    let rpc_id_owned = rpc_id.to_string();
    let opened_at = std::time::Instant::now();

    info!(
        surface = "api",
        service = "marketplace",
        action = "cherry_pick.progress.subscribe",
        rpc_id = %rpc_id_owned,
        "cherry-pick progress SSE stream opened"
    );

    let event_stream = stream::unfold(
        (receiver, rpc_id_owned, opened_at),
        move |(mut receiver, rpc_id, opened_at)| async move {
            loop {
                match receiver.recv().await {
                    Ok(frame) => match serde_json::to_string(&frame) {
                        Ok(payload) => {
                            return Some((
                                Ok(Event::default().data(payload)),
                                (receiver, rpc_id, opened_at),
                            ));
                        }
                        Err(error) => {
                            warn!(
                                error = %error,
                                rpc_id = %rpc_id,
                                "serialize progress frame for SSE"
                            );
                        }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(
                            skipped,
                            rpc_id = %rpc_id,
                            "cherry-pick SSE subscriber lagged"
                        );
                        return Some((
                            Ok(Event::default()
                                .event("lag")
                                .data(skipped.to_string())),
                            (receiver, rpc_id, opened_at),
                        ));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!(
                            surface = "api",
                            service = "marketplace",
                            action = "cherry_pick.progress.finish",
                            rpc_id = %rpc_id,
                            elapsed_ms = opened_at.elapsed().as_millis(),
                            "cherry-pick progress SSE stream closed (rpc complete)"
                        );
                        return None;
                    }
                }
            }
        },
    );

    Ok(Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)).text("keepalive")))
}
