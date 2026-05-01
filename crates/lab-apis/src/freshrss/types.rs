//! FreshRSS DTOs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FreshRssResponse {
    pub value: Value,
}
