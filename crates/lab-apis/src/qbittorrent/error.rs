//! qBittorrent-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `QbittorrentClient`.
#[derive(Debug, thiserror::Error)]
pub enum QbittorrentError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// qBittorrent returned "Fails." for a command (its custom error body).
    #[error("qBittorrent command failed: {0}")]
    CommandFailed(String),
}
