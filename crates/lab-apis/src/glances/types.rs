//! Glances DTOs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlancesResponse {
    pub value: Value,
}
