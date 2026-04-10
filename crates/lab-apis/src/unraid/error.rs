//! Error type for the `Unraid` service.

use crate::core::error::ApiError;

/// Errors returned by [`UnraidClient`](super::client::UnraidClient).
#[derive(Debug, thiserror::Error)]
pub enum UnraidError {
    /// Underlying HTTP / GraphQL transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] ApiError),

    /// Required environment variables are absent.
    #[error("not configured: UNRAID_URL and UNRAID_API_KEY required")]
    NotConfigured,
}
