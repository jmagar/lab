//! Scaffolded error type for adguard.

use crate::core::error::ApiError;

/// Scaffolded service error.
#[derive(Debug, thiserror::Error)]
pub enum AdguardError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
