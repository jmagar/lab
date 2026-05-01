//! Glances error type.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum GlancesError {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("invalid parameter: {0}")]
    InvalidParam(String),
}
