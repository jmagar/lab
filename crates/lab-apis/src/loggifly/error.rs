//! Scaffolded error type for loggifly.

use crate::core::error::ApiError;

/// Scaffolded service error.
#[derive(Debug, thiserror::Error)]
pub enum LoggiflyError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Local config or heartbeat file could not be inspected.
    #[error("io error: {0}")]
    Io(String),
}
