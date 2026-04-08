//! Qdrant-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `QdrantClient`.
#[derive(Debug, thiserror::Error)]
pub enum QdrantError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
