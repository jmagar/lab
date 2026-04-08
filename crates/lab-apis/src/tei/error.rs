//! TEI-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `TeiClient`.
#[derive(Debug, thiserror::Error)]
pub enum TeiError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
