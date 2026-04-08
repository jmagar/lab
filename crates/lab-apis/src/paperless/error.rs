//! Paperless-ngx-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `PaperlessClient`.
#[derive(Debug, thiserror::Error)]
pub enum PaperlessError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
