//! Error type for Uptime Kuma.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum UptimeKumaError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Socket.IO operation failed.
    #[error("socket.io error: {0}")]
    Socket(String),
    /// Login was rejected by Uptime Kuma.
    #[error("authentication failed: {0}")]
    Auth(String),
    /// Ack did not arrive before the timeout.
    #[error("Uptime Kuma ack timed out after {0}s")]
    Timeout(u64),
    /// Request parameter failed validation.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
    /// Live read is not available for this action.
    #[error("unsupported action: {0}")]
    Unsupported(String),
}
