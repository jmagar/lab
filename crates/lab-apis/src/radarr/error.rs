//! Radarr-specific errors.

use crate::core::error::ApiError;

/// Errors returned by `RadarrClient`.
///
/// `ApiError` covers transport, HTTP status, and JSON-decode failures in a
/// single uniform enum; service-specific variants wrap domain concepts that
/// don't fit the generic shape (e.g. "movie not found" when Radarr returns
/// 404 with a typed body).
#[derive(Debug, thiserror::Error)]
pub enum RadarrError {
    /// Upstream HTTP/transport error or non-success status.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// Radarr returned 404 for a resource lookup.
    #[error("radarr: {kind} with id {id} not found")]
    NotFound { kind: &'static str, id: i64 },
}
