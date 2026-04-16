use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

use crate::device::checkin::{DeviceHello, DeviceStatus};

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
}
