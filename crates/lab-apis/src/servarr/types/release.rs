//! Release (indexer search result) types.

use serde::{Deserialize, Serialize};

use super::protocol::DownloadProtocol;

/// One release returned from an indexer search.
///
/// Mirrors `ReleaseResource` from the Radarr v3 / Sonarr v3 `OpenAPI` specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    /// Stable per-indexer identifier.
    pub guid: String,
    /// Release title as advertised by the indexer.
    pub title: String,
    /// Indexer display name.
    pub indexer: String,
    /// Internal id of the indexer record.
    pub indexer_id: i64,
    /// Size in bytes.
    pub size: i64,
    /// Age in days.
    pub age: i32,
    /// Age in fractional hours (more precise than `age`).
    #[serde(default)]
    pub age_hours: f64,
    /// Transfer protocol.
    pub protocol: DownloadProtocol,
    /// Direct download URL (`.nzb` / `.torrent`), if exposed.
    pub download_url: Option<String>,
    /// Upstream release details page.
    pub info_url: Option<String>,
    /// Torrent seeders, if known.
    #[serde(default)]
    pub seeders: Option<i32>,
    /// Torrent leechers, if known.
    #[serde(default)]
    pub leechers: Option<i32>,
    /// Passed indexer approval.
    #[serde(default)]
    pub approved: bool,
    /// Rejected by an indexer filter.
    #[serde(default)]
    pub rejected: bool,
    /// Rejection reasons, one per filter that tripped.
    #[serde(default)]
    pub rejections: Vec<String>,
}
