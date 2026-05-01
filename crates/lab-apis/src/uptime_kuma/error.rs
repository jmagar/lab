//! Scaffolded error type for uptime_kuma.

use crate::core::error::ApiError;

/// Scaffolded service error.
#[derive(Debug, thiserror::Error)]
pub enum UptimeKumaError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
