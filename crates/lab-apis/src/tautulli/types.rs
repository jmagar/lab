//! Tautulli request/response types.

use serde::{Deserialize, Serialize};

/// Query parameters for `get_history`.
#[derive(Debug, Default)]
pub struct HistoryQuery {
    /// Page number (1-based).
    pub page: Option<u32>,
    /// Number of results per page (default: 25).
    pub page_size: Option<u32>,
    /// Order direction: `asc` or `desc`.
    pub order_dir: Option<String>,
    /// Media type filter: `movie`, `episode`, `track`.
    pub media_type: Option<String>,
    /// Filter by Plex user ID.
    pub user_id: Option<i64>,
    /// Filter by library section ID.
    pub section_id: Option<i64>,
    /// Filter by rating key (media item ID).
    pub rating_key: Option<i64>,
}

/// Query parameters for `get_user_ips`.
#[derive(Debug, Default)]
pub struct UserIpsQuery {
    /// Tautulli user ID.
    pub user_id: i64,
    /// Page number.
    pub page: Option<u32>,
    /// Page size.
    pub page_size: Option<u32>,
}

/// A single activity entry from the current sessions.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// Session key.
    pub session_key: Option<String>,
    /// Session ID.
    pub session_id: Option<String>,
    /// Username.
    pub username: Option<String>,
    /// User friendly name.
    pub friendly_name: Option<String>,
    /// Media type.
    pub media_type: Option<String>,
    /// Full title.
    pub full_title: Option<String>,
    /// Player name.
    pub player: Option<String>,
    /// Platform.
    pub platform: Option<String>,
    /// State (playing, paused, buffering).
    pub state: Option<String>,
    /// Progress percentage.
    pub progress_percent: Option<String>,
    /// IP address.
    pub ip_address: Option<String>,
    /// Bandwidth (kbps).
    pub bandwidth: Option<String>,
    /// Video decision (transcode, directplay, etc.).
    pub video_decision: Option<String>,
    /// Audio decision.
    pub audio_decision: Option<String>,
}

/// A Tautulli API response envelope.
#[derive(Debug, Deserialize)]
pub struct TautulliResponse<T> {
    /// Response status: "success" or "error".
    pub response: ResponseBody<T>,
}

/// Inner body of a Tautulli API response.
#[derive(Debug, Deserialize)]
pub struct ResponseBody<T> {
    /// "success" or "error".
    pub result: String,
    /// Response data.
    pub data: Option<T>,
    /// Error or info message.
    pub message: Option<String>,
}
