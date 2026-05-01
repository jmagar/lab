//! Error type for Uptime Kuma.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum UptimeKumaError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Requested action requires the Socket.IO actor that is not enabled yet.
    #[error("{0}")]
    Unsupported(String),
}
