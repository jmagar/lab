use std::sync::Arc;

use rmcp::RoleServer;
use rmcp::service::Peer;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

use crate::dispatch::gateway::types::GatewayCatalogDiff;

/// MCP-specific peer fanout that forwards catalog-change notifications to all
/// connected `rmcp::Peer<RoleServer>` instances.
///
/// This keeps `rmcp` types out of the dispatch layer while allowing
/// `GatewayManager` to notify peers when the upstream pool changes.
#[derive(Clone, Default)]
pub struct PeerNotifier {
    pub peers: Arc<RwLock<Vec<Peer<RoleServer>>>>,
}

impl PeerNotifier {
    pub async fn run(self, mut rx: mpsc::UnboundedReceiver<GatewayCatalogDiff>) {
        while let Some(diff) = rx.recv().await {
            self.notify_catalog_changes(&diff).await;
        }
    }

    async fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        let peers = self.peers.read().await.clone();
        let mut alive = Vec::with_capacity(peers.len());
        for (index, peer) in peers.iter().enumerate() {
            let peer = peer.clone();
            let mut ok = true;
            if diff.tools_changed {
                ok = peer.notify_tool_list_changed().await.is_ok();
                if !ok {
                    tracing::warn!(
                        peer_index = index,
                        phase = "tools",
                        tools_changed = diff.tools_changed,
                        resources_changed = diff.resources_changed,
                        prompts_changed = diff.prompts_changed,
                        "failed to notify peer about catalog change; pruning stale session"
                    );
                }
            }
            if ok && diff.resources_changed {
                ok = peer.notify_resource_list_changed().await.is_ok();
                if !ok {
                    tracing::warn!(
                        peer_index = index,
                        phase = "resources",
                        tools_changed = diff.tools_changed,
                        resources_changed = diff.resources_changed,
                        prompts_changed = diff.prompts_changed,
                        "failed to notify peer about catalog change; pruning stale session"
                    );
                }
            }
            if ok && diff.prompts_changed {
                ok = peer.notify_prompt_list_changed().await.is_ok();
                if !ok {
                    tracing::warn!(
                        peer_index = index,
                        phase = "prompts",
                        tools_changed = diff.tools_changed,
                        resources_changed = diff.resources_changed,
                        prompts_changed = diff.prompts_changed,
                        "failed to notify peer about catalog change; pruning stale session"
                    );
                }
            }
            if ok {
                alive.push(peer);
            }
        }

        let mut guard = self.peers.write().await;
        let added_since_snapshot = guard.split_off(peers.len());
        *guard = alive;
        guard.extend(added_since_snapshot);
    }
}
