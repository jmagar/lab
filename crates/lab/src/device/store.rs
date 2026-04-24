use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::log_event::DeviceLogEvent;

const MAX_LOG_EVENTS_PER_DEVICE: usize = 10_000;

#[derive(Debug, Clone, Default)]
pub struct DeviceFleetStore {
    inner: Arc<RwLock<BTreeMap<String, DeviceSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct DeviceSnapshot {
    pub node_id: String,
    pub connected: bool,
    pub last_seen: SystemTime,
    pub role: Option<String>,
    pub status: Option<DeviceStatus>,
    pub metadata: Option<DeviceMetadataUpload>,
    pub logs: Vec<DeviceLogEvent>,
}

impl DeviceFleetStore {
    pub async fn record_hello(&self, hello: DeviceHello) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(hello.node_id.clone())
            .or_insert_with(|| DeviceSnapshot {
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

    pub async fn record_status(&self, status: DeviceStatus) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(status.node_id.clone())
            .or_insert_with(|| DeviceSnapshot {
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
            .or_insert_with(|| DeviceSnapshot {
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

    pub async fn device(&self, node_id: &str) -> Option<DeviceSnapshot> {
        let inner = self.inner.read().await;
        inner.get(node_id).cloned()
    }

    pub async fn list_devices(&self) -> Vec<DeviceSnapshot> {
        let inner = self.inner.read().await;
        inner.values().cloned().collect()
    }

    pub async fn record_metadata(&self, metadata: DeviceMetadataUpload) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(metadata.node_id.clone())
            .or_insert_with(|| DeviceSnapshot {
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

    pub async fn record_logs(&self, node_id: &str, events: Vec<DeviceLogEvent>) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(node_id.to_string())
            .or_insert_with(|| DeviceSnapshot {
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
            .saturating_sub(MAX_LOG_EVENTS_PER_DEVICE);
        if excess > 0 {
            snapshot.logs.drain(0..excess);
        }
    }

    pub async fn search_logs_for_device(
        &self,
        node_id: &str,
        needle: &str,
        offset: usize,
        limit: usize,
    ) -> Vec<DeviceLogEvent> {
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
