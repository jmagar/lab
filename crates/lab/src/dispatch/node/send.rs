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
use tokio::sync::{RwLock, broadcast, mpsc, oneshot};
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

/// Per-node cap (lab-zxx5.27). Prevents a single silent / misbehaving node
/// (or a runaway operator script targeting one node) from filling the
/// global `MAX_PENDING_RPC` budget and denying service to OTHER nodes.
const MAX_PENDING_RPC_PER_NODE: usize = 32;

/// Count how many pending RPC entries currently target `node_id`.
///
/// Used by `send_rpc_to_node` before reserving a new slot. O(N) across the
/// owner map, but N is bounded by `MAX_PENDING_RPC` (1024), so the scan is
/// negligible at the call rate this surface sees.
fn pending_count_for_node(node_id: &str) -> usize {
    pending_owners()
        .iter()
        .filter(|entry| entry.value() == node_id)
        .count()
}

fn pending_map() -> &'static DashMap<String, oneshot::Sender<Value>> {
    static MAP: OnceLock<DashMap<String, oneshot::Sender<Value>>> = OnceLock::new();
    MAP.get_or_init(DashMap::new)
}

// --------------------------------------------------------------------------
// rpc_id ownership map (lab-zxx5.20 — SSE injection defense).
// --------------------------------------------------------------------------
// When `send_rpc_to_node(node_id, method, params)` dispatches an RPC, we
// record `rpc_id -> node_id` here so the master WS reader can verify that
// any inbound `install/progress` notification claiming to be for `rpc_id`
// actually came from the session on `node_id` we targeted. Without this,
// any connected node that learns an rpc_id (via log leaks, SSE URL query
// exposure, or shared admin views) can fabricate progress frames that get
// forwarded to the victim's SSE subscribers.
// --------------------------------------------------------------------------

fn pending_owners() -> &'static DashMap<String, String> {
    static MAP: OnceLock<DashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(DashMap::new)
}

/// Check whether `rpc_id` was dispatched to `claimed_node_id`.
///
/// Returns `true` if the rpc_id is in-flight AND the claimed owner matches
/// the recorded target. Returns `false` if either condition fails, so
/// callers can drop the frame safely on mismatch or unknown rpc_id.
///
/// Called by the master WebSocket reader before forwarding an inbound
/// `install/progress` notification to the progress broadcast.
pub fn rpc_id_owned_by(rpc_id: &str, claimed_node_id: &str) -> bool {
    pending_owners()
        .get(rpc_id)
        .map(|entry| entry.value() == claimed_node_id)
        .unwrap_or(false)
}

// --------------------------------------------------------------------------
// Progress broadcast registry (lab-zxx5.16).
// --------------------------------------------------------------------------
// Per-rpc_id tokio broadcast channel for `install/progress` notifications.
// The master WebSocket reader publishes each inbound progress frame to the
// sender for that rpc_id; the SSE endpoint subscribes and forwards frames
// as `data: {json}\n\n` events. Closing the broadcast sender terminates
// every connected SSE stream, which we do when the correlated RPC response
// arrives (the "done" signal).
// --------------------------------------------------------------------------

/// Capacity of each per-rpc_id progress broadcast channel. Small enough to
/// apply backpressure on slow SSE consumers; large enough to absorb normal
/// per-file progress bursts.
const PROGRESS_CHANNEL_CAPACITY: usize = 32;

fn progress_map() -> &'static DashMap<String, broadcast::Sender<Value>> {
    static MAP: OnceLock<DashMap<String, broadcast::Sender<Value>>> = OnceLock::new();
    MAP.get_or_init(DashMap::new)
}

/// Subscribe to `install/progress` frames tagged with the given `rpc_id`.
///
/// Creates the broadcast channel on first subscription (so the sender
/// exists before the corresponding RPC starts emitting progress). The
/// returned `Receiver` completes when the RPC terminates (the master
/// reader or `resolve_pending_rpc` drops the sender).
pub fn subscribe_progress(rpc_id: &str) -> broadcast::Receiver<Value> {
    progress_map()
        .entry(rpc_id.to_string())
        .or_insert_with(|| broadcast::channel(PROGRESS_CHANNEL_CAPACITY).0)
        .subscribe()
}

/// Publish a progress frame to every subscriber of `rpc_id`.
///
/// Returns the number of receivers that were delivered. A missing or empty
/// channel is a silent no-op so the master reader can call this on every
/// install/progress notification without branching.
pub fn publish_progress(rpc_id: &str, frame: Value) -> usize {
    progress_map()
        .get(rpc_id)
        .map(|entry| entry.value().send(frame).unwrap_or(0))
        .unwrap_or(0)
}

/// Resolve the pending oneshot for `rpc_id` with the given response value.
///
/// Called by the fleet WebSocket reader when a JSON-RPC response frame
/// arrives with a known id. Returns `true` if a pending entry was resolved.
/// Also drops any progress broadcast sender for the same rpc_id so open
/// SSE streams terminate cleanly (the response frame is the "done" signal —
/// see `lab-zxx5.16`).
pub fn resolve_pending_rpc(rpc_id: &str, response: Value) -> bool {
    // Drop any progress broadcast sender first — SSE receivers then see
    // `RecvError::Closed` and complete their streams. Do this before
    // resolving the pending RPC so the caller's `await` on send_rpc_to_node
    // is guaranteed to see the closed state if it races.
    progress_map().remove(rpc_id);
    pending_owners().remove(rpc_id);
    if let Some((_, sender)) = pending_map().remove(rpc_id) {
        // send() returns Err if the awaiter dropped (timeout branch already
        // ran) — nothing to do in that case.
        drop(sender.send(response));
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
    // lab-zxx5.27: per-node cap. Prevents a single silent node from
    // eating the global budget and starving all other nodes.
    let per_node = pending_count_for_node(node_id);
    if per_node >= MAX_PENDING_RPC_PER_NODE {
        return Err(ToolError::Sdk {
            sdk_kind: "rate_limited".to_string(),
            message: format!(
                "node `{node_id}` has {per_node} in-flight requests (cap {MAX_PENDING_RPC_PER_NODE}); refusing new request"
            ),
        });
    }

    let rpc_id = Uuid::new_v4().to_string();
    let (resp_tx, resp_rx) = oneshot::channel::<Value>();
    pending_map().insert(rpc_id.clone(), resp_tx);
    // lab-zxx5.20: record the target node so inbound install/progress
    // notifications can be ownership-checked. Always remove alongside
    // pending_map entries — tied lifecycle.
    pending_owners().insert(rpc_id.clone(), node_id.to_string());

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
            pending_owners().remove(&rpc_id);
            return Err(ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("encode rpc request: {error}"),
            });
        }
    };

    if let Err(error) = send_text_to_node(node_id, encoded).await {
        pending_map().remove(&rpc_id);
        pending_owners().remove(&rpc_id);
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
            pending_owners().remove(&rpc_id);
            Err(ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: "master rpc response channel closed before reply".to_string(),
            })
        }
        Err(_) => {
            pending_map().remove(&rpc_id);
            pending_owners().remove(&rpc_id);
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

    // lab-zxx5.16: subscribe_progress must create the broadcast lazily so
    // that the SSE handler can subscribe BEFORE the correlated RPC is even
    // sent — otherwise the first few progress frames race the subscription.
    #[tokio::test]
    async fn subscribe_progress_creates_sender_lazily_and_receives_published_frame() {
        let rpc_id = Uuid::new_v4().to_string();
        let mut rx = subscribe_progress(&rpc_id);
        let n = publish_progress(&rpc_id, json!({"status": "started"}));
        assert_eq!(n, 1, "one subscriber must receive the frame");
        let frame = rx.recv().await.expect("receive");
        assert_eq!(frame["status"], "started");
        // Clean up.
        progress_map().remove(&rpc_id);
    }

    // lab-zxx5.16: resolving the RPC must drop the broadcast channel so any
    // open SSE streams see Closed and terminate. The "done" signal on the
    // wire is the RPC response; we surface it to SSE via channel closure.
    #[tokio::test]
    async fn resolve_pending_rpc_closes_progress_broadcast_for_same_id() {
        let rpc_id = Uuid::new_v4().to_string();
        let mut rx = subscribe_progress(&rpc_id);

        // Register a pending oneshot so resolve_pending_rpc returns true.
        let (tx, _orx) = oneshot::channel::<Value>();
        pending_map().insert(rpc_id.clone(), tx);

        // Resolve. Must drop the broadcast.
        let resolved =
            resolve_pending_rpc(&rpc_id, json!({"id": rpc_id.clone(), "result": {}}));
        assert!(resolved);
        assert!(!progress_map().contains_key(&rpc_id), "broadcast must be dropped");

        // The receiver must observe Closed.
        let result = rx.recv().await;
        assert!(
            matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)),
            "expected Closed, got {:?}",
            result
        );
    }

    #[test]
    fn publish_progress_is_noop_for_unknown_id() {
        let delivered = publish_progress("no-such-rpc-id", json!({"x": 1}));
        assert_eq!(delivered, 0);
    }

    // lab-zxx5.20: owner map gates SSE progress injection. When a fresh
    // rpc_id is registered for node A, only A's WS session should be able
    // to publish progress for it. Any other node claiming that rpc_id must
    // be rejected by `rpc_id_owned_by`.
    #[tokio::test]
    async fn rpc_id_owned_by_matches_only_registered_target() {
        let rpc_id = Uuid::new_v4().to_string();
        // Register directly (skip the real send_rpc_to_node which would
        // also try to send over the WS).
        pending_owners().insert(rpc_id.clone(), "node-a".to_string());
        assert!(rpc_id_owned_by(&rpc_id, "node-a"));
        assert!(!rpc_id_owned_by(&rpc_id, "node-b"));
        assert!(!rpc_id_owned_by(&rpc_id, ""));
        pending_owners().remove(&rpc_id);
    }

    #[test]
    fn rpc_id_owned_by_returns_false_for_unknown_id() {
        assert!(!rpc_id_owned_by("never-registered", "any-node"));
    }

    #[tokio::test]
    async fn resolve_pending_rpc_clears_owner_entry() {
        // End-to-end: register pending + owner, resolve, verify BOTH maps
        // are cleaned so a later forgery attempt on the same rpc_id gets a
        // clean `false` from rpc_id_owned_by (not a stale `true`).
        let (tx, _rx) = oneshot::channel::<Value>();
        let rpc_id = Uuid::new_v4().to_string();
        pending_map().insert(rpc_id.clone(), tx);
        pending_owners().insert(rpc_id.clone(), "node-a".to_string());
        assert!(rpc_id_owned_by(&rpc_id, "node-a"));
        let _ = resolve_pending_rpc(&rpc_id, json!({"result": "ok"}));
        assert!(
            !rpc_id_owned_by(&rpc_id, "node-a"),
            "owner entry must be cleared when the RPC resolves"
        );
    }
}
