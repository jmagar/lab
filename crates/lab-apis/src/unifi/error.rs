//! UniFi-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `UnifiClient`.
#[derive(Debug, thiserror::Error)]
pub enum UnifiError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
