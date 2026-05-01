//! Jellyfin error type.

use crate::core::error::ApiError;

/// Error returned by the Jellyfin client.
#[derive(Debug, thiserror::Error)]
pub enum JellyfinError {
    /// Shared transport or response error.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// Client-side parameter validation failed.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
}
