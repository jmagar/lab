//! Plex-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `PlexClient`.
#[derive(Debug, thiserror::Error)]
pub enum PlexError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
