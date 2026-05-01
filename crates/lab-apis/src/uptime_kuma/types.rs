//! Uptime Kuma response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generic JSON wrapper returned by contract/read actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UptimeKumaResponse {
    pub value: Value,
}
