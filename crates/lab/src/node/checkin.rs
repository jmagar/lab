use serde::{Deserialize, Serialize};

use crate::node::config_scan::DiscoveredMcpConfigFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHello {
    #[serde(alias = "device_id")]
    pub node_id: String,
    pub role: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub connected: bool,
    pub cpu_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub storage_used_bytes: Option<u64>,
    pub os: Option<String>,
    pub ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadataUpload {
    pub node_id: String,
    pub discovered_configs: Vec<DiscoveredMcpConfigFile>,
}
