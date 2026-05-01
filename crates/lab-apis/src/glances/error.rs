//! Scaffolded error type for glances.

use crate::core::error::ApiError;

/// Scaffolded service error.
#[derive(Debug, thiserror::Error)]
pub enum GlancesError {
    /// Upstream HTTP/transport error.
    #[error(transparent)]
    Api(#[from] ApiError),
}
