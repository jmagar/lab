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
        tracing::info!(
            surface = "mcp",
            service = "peers",
            action = "notifier.start",
            subsystem = "mcp_server",
            phase = "peer_notifier.start",
            "starting MCP peer catalog-change notifier"
        );
        while let Some(diff) = rx.recv().await {
            self.notify_catalog_changes(&diff).await;
        }
        tracing::info!(
            surface = "mcp",
            service = "peers",
            action = "notifier.stop",
            subsystem = "mcp_server",
            phase = "peer_notifier.stop",
            "MCP peer catalog-change notifier stopped"
        );
    }

    async fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        let peers = self.peers.read().await.clone();
        tracing::info!(
            surface = "mcp",
            service = "peers",
            action = "catalog.notify",
            subsystem = "mcp_server",
            phase = "catalog.notify",
            peer_count = peers.len(),
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            "broadcasting catalog change to connected peers"
        );
        let mut alive = Vec::with_capacity(peers.len());
        for (index, peer) in peers.iter().enumerate() {
            let peer = peer.clone();
            let mut ok = true;
            if diff.tools_changed {
                ok = peer.notify_tool_list_changed().await.is_ok();
                if !ok {
                    tracing::warn!(
                        surface = "mcp",
                        service = "peers",
                        action = "peer.disconnect",
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
                        surface = "mcp",
                        service = "peers",
                        action = "peer.disconnect",
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
                        surface = "mcp",
                        service = "peers",
                        action = "peer.disconnect",
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

        let pruned = peers.len().saturating_sub(alive.len());
        let mut guard = self.peers.write().await;
        let added_since_snapshot = guard.split_off(peers.len());
        *guard = alive;
        guard.extend(added_since_snapshot);
        let total = guard.len();
        if pruned > 0 {
            tracing::info!(
                surface = "mcp",
                service = "peers",
                action = "peer.gc",
                pruned_count = pruned,
                active_count = total,
                "pruned stale MCP peer sessions after catalog notify",
            );
        } else {
            tracing::debug!(
                surface = "mcp",
                service = "peers",
                action = "peer.gc",
                active_count = total,
                "catalog notify complete — all peers alive",
            );
        }
    }
}
