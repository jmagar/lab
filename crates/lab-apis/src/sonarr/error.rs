//! Sonarr-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `SonarrClient`.
#[derive(Debug, thiserror::Error)]
pub enum SonarrError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
