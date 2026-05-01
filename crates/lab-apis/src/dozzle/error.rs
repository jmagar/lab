//! Dozzle-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `DozzleClient`.
#[derive(Debug, thiserror::Error)]
pub enum DozzleError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Upstream returned data in a shape Lab cannot consume safely.
    #[error("invalid Dozzle response: {0}")]
    InvalidResponse(String),
    /// A bounded stream did not produce the required event in time.
    #[error("Dozzle stream timed out before required event: {0}")]
    StreamTimeout(String),
}
