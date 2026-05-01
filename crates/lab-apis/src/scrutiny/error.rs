//! Scrutiny error type.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum ScrutinyError {
    #[error(transparent)]
    Api(#[from] ApiError),
}
