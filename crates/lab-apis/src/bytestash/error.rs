//! ByteStash-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `ByteStashClient`.
#[derive(Debug, thiserror::Error)]
pub enum ByteStashError {
    /// Upstream HTTP/transport error or non-success status.
    #[error(transparent)]
    Api(#[from] ApiError),
}
