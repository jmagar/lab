//! Overseerr-specific error type.

use thiserror::Error;

use crate::core::ApiError;

/// Errors returned by [`super::client::OverseerrClient`].
#[derive(Debug, Error)]
pub enum OverseerrError {
    /// Underlying transport or HTTP error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
