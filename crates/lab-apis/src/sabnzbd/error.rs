//! SABnzbd-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `SabnzbdClient`.
#[derive(Debug, thiserror::Error)]
pub enum SabnzbdError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
