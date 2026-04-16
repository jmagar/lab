use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::log_event::DeviceLogEvent;

#[derive(Debug, Clone, Default)]
pub struct DeviceFleetStore {
    inner: Arc<RwLock<BTreeMap<String, DeviceSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct DeviceSnapshot {
    pub device_id: String,
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
            .entry(hello.device_id.clone())
            .or_insert_with(|| DeviceSnapshot {
                device_id: hello.device_id.clone(),
                connected: false,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.device_id = hello.device_id;
        snapshot.last_seen = SystemTime::now();
        snapshot.role = Some(hello.role);
    }

    pub async fn record_status(&self, status: DeviceStatus) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(status.device_id.clone())
            .or_insert_with(|| DeviceSnapshot {
                device_id: status.device_id.clone(),
                connected: status.connected,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.device_id = status.device_id.clone();
        snapshot.connected = status.connected;
        snapshot.last_seen = SystemTime::now();
        snapshot.status = Some(status);
    }

    pub async fn device(&self, device_id: &str) -> Option<DeviceSnapshot> {
        let inner = self.inner.read().await;
        inner.get(device_id).cloned()
    }

    pub async fn list_devices(&self) -> Vec<DeviceSnapshot> {
        let inner = self.inner.read().await;
        inner.values().cloned().collect()
    }

    pub async fn record_metadata(&self, metadata: DeviceMetadataUpload) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(metadata.device_id.clone())
            .or_insert_with(|| DeviceSnapshot {
                device_id: metadata.device_id.clone(),
                connected: false,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.device_id = metadata.device_id.clone();
        snapshot.last_seen = SystemTime::now();
        snapshot.metadata = Some(metadata);
    }

    pub async fn record_logs(&self, device_id: &str, events: Vec<DeviceLogEvent>) {
        let mut inner = self.inner.write().await;
        let snapshot = inner
            .entry(device_id.to_string())
            .or_insert_with(|| DeviceSnapshot {
                device_id: device_id.to_string(),
                connected: false,
                last_seen: SystemTime::now(),
                role: None,
                status: None,
                metadata: None,
                logs: Vec::new(),
            });
        snapshot.last_seen = SystemTime::now();
        snapshot.logs.extend(events);
    }

    pub async fn logs_for_device(&self, device_id: &str) -> Vec<DeviceLogEvent> {
        let inner = self.inner.read().await;
        inner
            .get(device_id)
            .map(|snapshot| snapshot.logs.clone())
            .unwrap_or_default()
    }
}
