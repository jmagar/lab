//! Memos-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `MemosClient`.
#[derive(Debug, thiserror::Error)]
pub enum MemosError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
