//! Apprise-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `AppriseClient`.
#[derive(Debug, thiserror::Error)]
pub enum AppriseError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
