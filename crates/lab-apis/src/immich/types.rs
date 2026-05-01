//! Immich response DTOs exposed by Lab.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMe {
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchRequest {
    pub query: Option<String>,
    pub limit: u32,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchResponse {
    pub items: Vec<Value>,
    pub page: Option<u32>,
    pub next_page: Option<u32>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub value: Value,
}
