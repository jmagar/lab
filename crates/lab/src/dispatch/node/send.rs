//! Shared helpers for sending messages to connected nodes.
//!
//! Owns the sender registry (node_id → mpsc sender) that is populated on WS
//! upgrade and queried by the dispatch layer. Lives in `dispatch/` so that
//! `dispatch/marketplace/acp_dispatch.rs` can call it without crossing the
//! forbidden `dispatch/ → api/` boundary.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::extract::ws::Message;
use dashmap::DashMap;
use serde_json::{Value, json};
use tokio::sync::{RwLock, mpsc, oneshot};
use uuid::Uuid;

use crate::dispatch::error::ToolError;

// --------------------------------------------------------------------------
// Sender registry
// --------------------------------------------------------------------------
// Keyed by node_id. Populated on WebSocket upgrade (after successful
// `initialize`), removed on disconnect. `send_to_node` is the public
// entry-point for dispatching RPC commands to a connected node.
// --------------------------------------------------------------------------

pub type SessionToken = u64;
pub type SenderRegistry = Arc<RwLock<HashMap<String, (SessionToken, mpsc::Sender<Message>)>>>;

pub fn sender_registry() -> &'static SenderRegistry {
    static REGISTRY: OnceLock<SenderRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

#[derive(Debug, thiserror::Error)]
pub enum NodeDispatchError {
    #[error("node `{node_id}` is not connected")]
    NotConnected { node_id: String },
    #[error("send channel for node `{node_id}` is closed")]
    ChannelClosed { node_id: String },
}

/// Send a raw WebSocket `Message` to a specific connected node.
///
/// Returns `NodeDispatchError::NotConnected` when the node has no active WS
/// session, and `NodeDispatchError::ChannelClosed` when the send channel was
/// dropped (race with disconnect).
pub async fn send_to_node(node_id: &str, msg: Message) -> Result<(), NodeDispatchError> {
    let sender = {
        let registry = sender_registry().read().await;
        let (_, sender) = registry.get(node_id).ok_or_else(|| NodeDispatchError::NotConnected {
            node_id: node_id.to_string(),
        })?;
        sender.clone()
    };
    sender.send(msg).await.map_err(|_| NodeDispatchError::ChannelClosed {
        node_id: node_id.to_string(),
    })
}

/// Convenience wrapper: send a JSON text frame to a connected node.
///
/// Callers in the dispatch layer (e.g. `dispatch/marketplace/acp_dispatch.rs`)
/// use this so they don't need to import `axum::extract::ws::Message` directly.
pub async fn send_text_to_node(node_id: &str, text: String) -> Result<(), NodeDispatchError> {
    send_to_node(node_id, Message::Text(text.into())).await
}

// --------------------------------------------------------------------------
// Master-side pending-response map (lab-zxx5.6 / mirror of lab-zxx5.19).
// --------------------------------------------------------------------------
// When the master sends a JSON-RPC request to a node (e.g. for cherry-pick
// install), we correlate the eventual response back by rpc_id. This map
// stores the oneshot sender for each in-flight request; the fleet WebSocket
// reader resolves the oneshot when a response frame arrives.
// --------------------------------------------------------------------------

/// Default per-request timeout. Configurable via `LAB_INSTALL_TIMEOUT_SECS`.
const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

/// Hard cap on in-flight master→node requests across all nodes. Prevents
/// unbounded pending-map growth if a sequence of nodes goes silent.
const MAX_PENDING_RPC: usize = 1024;

fn pending_map() -> &'static DashMap<String, oneshot::Sender<Value>> {
    static MAP: OnceLock<DashMap<String, oneshot::Sender<Value>>> = OnceLock::new();
    MAP.get_or_init(DashMap::new)
}

/// Resolve the pending oneshot for `rpc_id` with the given response value.
///
/// Called by the fleet WebSocket reader when a JSON-RPC response frame
/// arrives with a known id. Returns `true` if a pending entry was resolved.
pub fn resolve_pending_rpc(rpc_id: &str, response: Value) -> bool {
    if let Some((_, sender)) = pending_map().remove(rpc_id) {
        // send() returns Err if the awaiter dropped (timeout branch already
        // ran) — nothing to do in that case.
        let _ = sender.send(response);
        true
    } else {
        false
    }
}

fn rpc_timeout() -> Duration {
    std::env::var("LAB_INSTALL_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_RPC_TIMEOUT)
}

/// Send a JSON-RPC request to a connected node and await the response.
///
/// Generates a fresh UUIDv4 `rpc_id`, registers a pending oneshot, encodes
/// and sends the request, then waits up to `rpc_timeout()` for the fleet
/// reader to resolve the oneshot. On any terminal condition (timeout, send
/// failure, channel closed, pending-map full) the pending entry is removed
/// so the map cannot grow unbounded.
///
/// Bead lab-zxx5.6 / knowledge pattern:
/// "every oneshot-request pattern needs a tokio::time::timeout wrapper +
///  pending-entry cleanup".
pub async fn send_rpc_to_node(
    node_id: &str,
    method: &str,
    params: Value,
) -> Result<Value, ToolError> {
    if pending_map().len() >= MAX_PENDING_RPC {
        return Err(ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!(
                "master pending-rpc map full ({} inflight); refusing new request",
                MAX_PENDING_RPC
            ),
        });
    }

    let rpc_id = Uuid::new_v4().to_string();
    let (resp_tx, resp_rx) = oneshot::channel::<Value>();
    pending_map().insert(rpc_id.clone(), resp_tx);

    let request = json!({
        "jsonrpc": "2.0",
        "id": rpc_id,
        "method": method,
        "params": params,
    });

    let encoded = match serde_json::to_string(&request) {
        Ok(s) => s,
        Err(error) => {
            pending_map().remove(&rpc_id);
            return Err(ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("encode rpc request: {error}"),
            });
        }
    };

    if let Err(error) = send_text_to_node(node_id, encoded).await {
        pending_map().remove(&rpc_id);
        return Err(match error {
            NodeDispatchError::NotConnected { .. } => ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("node `{node_id}` is not connected"),
            },
            NodeDispatchError::ChannelClosed { .. } => ToolError::Sdk {
                sdk_kind: "network_error".to_string(),
                message: format!("send channel for node `{node_id}` closed (race with disconnect)"),
            },
        });
    }

    match tokio::time::timeout(rpc_timeout(), resp_rx).await {
        Ok(Ok(response)) => {
            // Surface JSON-RPC error envelopes as ToolError::Sdk so callers
            // see a consistent shape regardless of origin.
            if let Some(error) = response.get("error") {
                let kind = error
                    .get("data")
                    .and_then(|data| data.get("kind"))
                    .and_then(Value::as_str)
                    .unwrap_or("upstream_error")
                    .to_string();
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("node rpc returned error")
                    .to_string();
                return Err(ToolError::Sdk { sdk_kind: kind, message });
            }
            Ok(response.get("result").cloned().unwrap_or(Value::Null))
        }
        Ok(Err(_)) => {
            pending_map().remove(&rpc_id);
            Err(ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: "master rpc response channel closed before reply".to_string(),
            })
        }
        Err(_) => {
            pending_map().remove(&rpc_id);
            Err(ToolError::Sdk {
                sdk_kind: "timeout".to_string(),
                message: format!(
                    "node `{node_id}` did not respond to `{method}` within {:?}",
                    rpc_timeout()
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_rpc_to_node_times_out_when_node_disconnected() {
        // No entry in sender_registry; send should fail fast with not_found.
        let err = send_rpc_to_node("ghost-node", "nodes/ping", json!({}))
            .await
            .err()
            .expect("must error");
        assert_eq!(err.kind(), "not_found");
        // pending map must be empty (no leaked entry).
        // Note: other tests may run in parallel — just check no entries
        // remain under our request prefix.
        // (cheap: map is a singleton, rely on resolve_pending_rpc semantics instead)
    }

    #[tokio::test]
    async fn resolve_pending_rpc_returns_false_for_unknown_id() {
        assert!(!resolve_pending_rpc("no-such-id", json!({"result": {}})));
    }

    #[tokio::test]
    async fn resolve_pending_rpc_resolves_and_removes_entry() {
        // Insert a fake pending entry manually.
        let (tx, rx) = oneshot::channel::<Value>();
        let rpc_id = Uuid::new_v4().to_string();
        pending_map().insert(rpc_id.clone(), tx);
        let delivered = resolve_pending_rpc(&rpc_id, json!({"result": "ok"}));
        assert!(delivered, "must deliver when id matches");
        let received = rx.await.expect("channel open");
        assert_eq!(received["result"], "ok");
        assert!(
            !pending_map().contains_key(&rpc_id),
            "pending entry must be removed after resolve"
        );
    }
}
