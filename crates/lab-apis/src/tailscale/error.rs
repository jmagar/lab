//! Tailscale-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `TailscaleClient`.
#[derive(Debug, thiserror::Error)]
pub enum TailscaleError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
