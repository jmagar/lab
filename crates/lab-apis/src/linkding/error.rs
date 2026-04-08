//! Linkding-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `LinkdingClient`.
#[derive(Debug, thiserror::Error)]
pub enum LinkdingError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
