//! OpenAI-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `OpenAiClient`.
#[derive(Debug, thiserror::Error)]
pub enum OpenAiError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
