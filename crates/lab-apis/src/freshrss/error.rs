//! FreshRSS error type.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum FreshrssError {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("invalid parameter: {0}")]
    InvalidParam(String),

    #[error("authentication response did not contain an auth token")]
    MissingAuthToken,
}
