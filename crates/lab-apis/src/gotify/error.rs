//! Gotify-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `GotifyClient`.
#[derive(Debug, thiserror::Error)]
pub enum GotifyError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
