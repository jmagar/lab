//! Prowlarr-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `ProwlarrClient`.
#[derive(Debug, thiserror::Error)]
pub enum ProwlarrError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
