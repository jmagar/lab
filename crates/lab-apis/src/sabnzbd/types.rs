//! SABnzbd request/response types.
//!
//! SABnzbd's API returns JSON when `output=json` is set. Responses wrap
//! data in varying top-level keys depending on the `mode`.

use serde::{Deserialize, Serialize};

/// SABnzbd version response (`mode=version`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

/// Top-level queue response (`mode=queue`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueResponse {
    pub queue: QueueInfo,
}

/// Queue metadata and slot list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub status: String,
    pub speed: String,
    #[serde(rename = "speedlimit")]
    pub speed_limit: String,
    #[serde(rename = "speedlimit_abs")]
    pub speed_limit_abs: String,
    pub paused: bool,
    #[serde(rename = "noofslots_total")]
    pub total_slots: u64,
    #[serde(rename = "mb")]
    pub total_mb: String,
    #[serde(rename = "mbleft")]
    pub mb_left: String,
    #[serde(rename = "timeleft")]
    pub time_left: String,
    pub slots: Vec<QueueSlot>,
}

/// A single item in the download queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSlot {
    pub nzo_id: String,
    pub filename: String,
    pub status: String,
    pub percentage: String,
    pub mb: String,
    pub mbleft: String,
    pub timeleft: String,
    pub cat: String,
    pub priority: String,
    #[serde(default)]
    pub size: String,
}

/// Top-level history response (`mode=history`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub history: HistoryInfo,
}

/// History metadata and slot list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryInfo {
    #[serde(rename = "noofslots")]
    pub total_slots: u64,
    pub slots: Vec<HistorySlot>,
}

/// A single item in download history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySlot {
    pub nzo_id: String,
    pub name: String,
    pub status: String,
    pub category: String,
    pub size: String,
    #[serde(default)]
    pub storage: String,
    #[serde(default)]
    pub completed: u64,
    #[serde(default)]
    pub download_time: u64,
    #[serde(default)]
    pub fail_message: String,
}

/// Server stats response (`mode=server_stats`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub total: u64,
    pub month: u64,
    pub week: u64,
    pub day: u64,
    #[serde(default)]
    pub servers: serde_json::Value,
}

/// Status response for pause/resume/speed operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: bool,
}

/// Warnings list (`mode=warnings`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningsResponse {
    pub warnings: Vec<String>,
}
