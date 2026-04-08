//! UniFi request/response types.

use serde::{Deserialize, Serialize};

/// General application info returned by `/v1/info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    #[serde(rename = "applicationVersion")]
    pub application_version: String,
}
