//! Tautulli-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `TautulliClient`.
#[derive(Debug, thiserror::Error)]
pub enum TautulliError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
