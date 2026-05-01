//! OpenACP-specific errors.

use crate::core::error::ApiError;

/// Errors returned by [`crate::openacp::OpenAcpClient`].
#[derive(Debug, thiserror::Error)]
pub enum OpenAcpError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
