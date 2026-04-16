use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLogEvent {
    pub device_id: String,
    pub source: String,
    pub timestamp_unix_ms: i64,
    pub level: Option<String>,
    pub message: String,
    pub fields: serde_json::Map<String, serde_json::Value>,
}
