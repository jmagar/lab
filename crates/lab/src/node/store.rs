use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

use crate::node::checkin::{NodeHello, NodeMetadataUpload, NodeStatus};
use crate::node::log_event::NodeLogEvent;

const MAX_LOG_EVENTS_PER_NODE: usize = 10_000;

#[derive(Debug, Clone, Default)]
pub struct NodeStore {
    inner: Arc<RwLock<BTreeMap<String, NodeSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct NodeSnapshot {
    pub node_id: String,
    pub connected: bool,
    pub last_seen: SystemTime,
    pub role: Option<String>,
    pub status: Option<NodeStatus>,
    pub metadata: Option<NodeMetadataUpload>,
    pub logs: Vec<NodeLogEvent>,
}

impl NodeStore {
    pub async fn record_hello(&self, hello: NodeHello) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(hello.node_id.clone())
            .or_insert_with(|| NodeSnapshot {
                node_id: hello.node_id.clone(),
                connected: true,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.node_id = hello.node_id;
        snapshot.connected = true;
        snapshot.last_seen = SystemTime::now();
        snapshot.role = Some(hello.role);
    }

    pub async fn record_status(&self, status: NodeStatus) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(status.node_id.clone())
            .or_insert_with(|| NodeSnapshot {
                node_id: status.node_id.clone(),
                connected: status.connected,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.node_id = status.node_id.clone();
        snapshot.connected = status.connected;
        snapshot.last_seen = SystemTime::now();
        snapshot.status = Some(status);
    }

    pub async fn set_connected(&self, node_id: &str, connected: bool) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(node_id.to_string())
            .or_insert_with(|| NodeSnapshot {
                node_id: node_id.to_string(),
                connected,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.connected = connected;
        snapshot.last_seen = SystemTime::now();
        if let Some(status) = snapshot.status.as_mut() {
            status.connected = connected;
        }
    }

    pub async fn node(&self, node_id: &str) -> Option<NodeSnapshot> {
        let inner = self.inner.read().await;
        inner.get(node_id).cloned()
    }

    pub async fn list_nodes(&self) -> Vec<NodeSnapshot> {
        let inner = self.inner.read().await;
        inner.values().cloned().collect()
    }

    pub async fn record_metadata(&self, metadata: NodeMetadataUpload) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(metadata.node_id.clone())
            .or_insert_with(|| NodeSnapshot {
                node_id: metadata.node_id.clone(),
                connected: false,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.node_id = metadata.node_id.clone();
        snapshot.last_seen = SystemTime::now();
        snapshot.metadata = Some(metadata);
    }

    pub async fn record_logs(&self, node_id: &str, events: Vec<NodeLogEvent>) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(node_id.to_string())
            .or_insert_with(|| NodeSnapshot {
                node_id: node_id.to_string(),
                connected: false,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.last_seen = SystemTime::now();
        snapshot.logs.extend(events);
        let excess = snapshot
            .logs
            .len()
            .saturating_sub(MAX_LOG_EVENTS_PER_NODE);
        if excess > 0 {
            snapshot.logs.drain(0..excess);
        }
    }

    pub async fn search_logs_for_node(
        &self,
        node_id: &str,
        needle: &str,
        offset: usize,
        limit: usize,
    ) -> Vec<NodeLogEvent> {
        let inner = self.inner.read().await;
        let Some(snapshot) = inner.get(node_id) else {
            return Vec::new();
        };

        let normalized_needle = needle.to_ascii_lowercase();
        let effective_limit = limit.max(1).min(1_000);
        snapshot
            .logs
            .iter()
            .filter(|event| {
                event
                    .message
                    .to_ascii_lowercase()
                    .contains(&normalized_needle)
            })
            .skip(offset)
            .take(effective_limit)
            .cloned()
            .collect()
    }
}
