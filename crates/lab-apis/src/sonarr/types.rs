//! Sonarr request/response types.
//!
//! TODO: add series, episode, queue, and season types from the Sonarr API v3 spec.

use serde::{Deserialize, Serialize};

/// A Sonarr TV series entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub overview: Option<String>,
    pub path: String,
    pub monitored: bool,
    /// Number of seasons. Sonarr API v3 serializes this as `"seasonCount"`.
    #[serde(rename = "seasonCount")]
    pub season_count: u32,
}
