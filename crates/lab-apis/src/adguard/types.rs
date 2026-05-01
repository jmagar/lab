//! AdGuard Home DTOs exposed by Lab.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdguardResponse {
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogResponse {
    pub value: Value,
    pub limit: u32,
}
