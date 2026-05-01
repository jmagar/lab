//! Navidrome DTOs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubsonicResponse {
    #[serde(rename = "subsonic-response")]
    pub response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub value: Value,
}
