//! qBittorrent request/response types.
//!
//! Derived from the qBittorrent WebUI API v2:
//! <https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)>

use serde::{Deserialize, Serialize};

/// A torrent entry from the torrent list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    /// Torrent info hash (v1).
    pub hash: String,
    /// Torrent name.
    pub name: String,
    /// Torrent state (e.g. `downloading`, `seeding`, `paused`, `error`).
    pub state: String,
    /// Total size in bytes.
    pub size: i64,
    /// Download progress [0.0, 1.0].
    pub progress: f64,
    /// Download speed in bytes/s.
    pub dlspeed: i64,
    /// Upload speed in bytes/s.
    pub upspeed: i64,
    /// Number of seeds connected.
    pub num_seeds: i32,
    /// Number of peers connected.
    pub num_leechs: i32,
    /// Ratio.
    pub ratio: f64,
    /// Estimated time of arrival in seconds (-1 = unknown).
    pub eta: i64,
    /// Category assigned to this torrent.
    #[serde(default)]
    pub category: String,
    /// Save path.
    #[serde(default)]
    pub save_path: String,
    /// Download completed bytes.
    pub downloaded: i64,
    /// Upload total bytes.
    pub uploaded: i64,
    /// Amount of data downloaded this session.
    #[serde(default)]
    pub downloaded_session: i64,
    /// Amount of data uploaded this session.
    #[serde(default)]
    pub uploaded_session: i64,
    /// Time when this torrent was added (Unix timestamp).
    pub added_on: i64,
    /// Time when download was completed (Unix timestamp, 0 if not complete).
    pub completion_on: i64,
    /// Magnet URI (may be empty if not available).
    #[serde(default)]
    pub magnet_uri: String,
}

/// Global transfer info returned by `/api/v2/transfer/info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    /// Global download speed (bytes/s).
    pub dl_info_speed: i64,
    /// Global download data (bytes).
    pub dl_info_data: i64,
    /// Global upload speed (bytes/s).
    pub up_info_speed: i64,
    /// Global upload data (bytes).
    pub up_info_data: i64,
    /// Download speed limit (bytes/s; 0 = unlimited).
    pub dl_rate_limit: i64,
    /// Upload speed limit (bytes/s; 0 = unlimited).
    pub up_rate_limit: i64,
    /// DHT nodes.
    pub dht_nodes: i64,
    /// Connection status (`connected`, `firewalled`, `disconnected`).
    pub connection_status: String,
}

/// Application preferences (partial — common fields only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Default save path for new torrents.
    pub save_path: String,
    /// Global download speed limit (bytes/s; 0 = unlimited).
    pub dl_limit: i64,
    /// Global upload speed limit (bytes/s; 0 = unlimited).
    pub up_limit: i64,
    /// Max active torrents (-1 = unlimited).
    pub max_active_torrents: i64,
    /// Max active downloads (-1 = unlimited).
    pub max_active_downloads: i64,
    /// Max active uploads (-1 = unlimited).
    pub max_active_uploads: i64,
    /// Alternative global download speed limit (bytes/s).
    pub alt_dl_limit: i64,
    /// Alternative global upload speed limit (bytes/s).
    pub alt_up_limit: i64,
    /// Whether the alternative rate limit is active.
    pub alt_global_speed_limit_enabled: bool,
}

/// Torrent properties returned by `/api/v2/torrents/properties`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentProperties {
    /// Save path.
    pub save_path: String,
    /// Creation date (Unix timestamp).
    pub creation_date: i64,
    /// Piece size in bytes.
    pub piece_size: i64,
    /// Torrent comment.
    pub comment: String,
    /// Total bytes wasted.
    pub total_wasted: i64,
    /// Total bytes uploaded.
    pub total_uploaded: i64,
    /// Total bytes downloaded.
    pub total_downloaded: i64,
    /// Upload speed (bytes/s).
    pub up_speed: i64,
    /// Download speed (bytes/s).
    pub dl_speed: i64,
    /// Elapsed time (seconds).
    pub time_elapsed: i64,
    /// Seeding time (seconds).
    pub seeding_time: i64,
    /// Number of connections.
    pub nb_connections: i64,
    /// Share ratio.
    pub share_ratio: f64,
    /// Addition time (Unix timestamp).
    pub addition_date: i64,
    /// Completion time (Unix timestamp; 0 if incomplete).
    pub completion_date: i64,
    /// Torrent creator.
    pub created_by: String,
    /// Download speed limit (bytes/s; -1 = global, 0 = unlimited).
    pub dl_speed_limit: i64,
    /// Upload speed limit (bytes/s; -1 = global, 0 = unlimited).
    pub up_speed_limit: i64,
}

/// Category entry from `/api/v2/torrents/categories`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Category name.
    pub name: String,
    /// Default save path for this category (empty = use default).
    #[serde(rename = "savePath")]
    pub save_path: String,
}

/// qBittorrent application version response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersion {
    /// Application version string (e.g. `v4.6.3`).
    pub version: String,
}

/// Log message entry from `/api/v2/log/main`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Message ID.
    pub id: i64,
    /// Log message text.
    pub message: String,
    /// Unix timestamp of log entry.
    pub timestamp: i64,
    /// Message type (1=normal, 2=info, 4=warning, 8=critical).
    #[serde(rename = "type")]
    pub message_type: i32,
}

/// A file entry within a torrent from `/api/v2/torrents/files`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    /// File index within the torrent.
    pub index: i64,
    /// File name (relative path within torrent).
    pub name: String,
    /// File size in bytes.
    pub size: i64,
    /// Download progress [0.0, 1.0].
    pub progress: f64,
    /// File priority (0=do not download, 1=normal, 6=high, 7=maximum).
    pub priority: i32,
    /// True if this is a seed file.
    #[serde(rename = "is_seed", default)]
    pub is_seed: Option<bool>,
    /// Piece range: [first_piece, last_piece] (inclusive).
    #[serde(rename = "piece_range")]
    pub piece_range: Vec<i64>,
    /// Download availability [0.0, 1.0].
    pub availability: f64,
}

/// Tracker entry from `/api/v2/torrents/trackers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    /// Tracker URL.
    pub url: String,
    /// Tracker status code.
    pub status: i32,
    /// Tier.
    pub tier: i32,
    /// Number of peers this tracker knows about.
    pub num_peers: i32,
    /// Number of seeds this tracker knows about.
    pub num_seeds: i32,
    /// Number of leeches this tracker knows about.
    pub num_leeches: i32,
    /// Number of downloads this tracker knows about.
    pub num_downloaded: i32,
    /// Message from tracker.
    pub msg: String,
}
