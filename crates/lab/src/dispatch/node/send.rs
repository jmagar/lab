//! Shared helpers for sending messages to connected nodes.
//!
//! Owns the sender registry (node_id → mpsc sender) that is populated on WS
//! upgrade and queried by the dispatch layer. Lives in `dispatch/` so that
//! `dispatch/marketplace/acp_dispatch.rs` can call it without crossing the
//! forbidden `dispatch/ → api/` boundary.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};

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
