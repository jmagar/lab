//! Error type for Pi-hole.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum PiholeError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Request parameter failed client-side validation.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
}
