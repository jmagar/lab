//! Shared HTTP client.
//!
//! Centralizes retries, exponential backoff, rate limiting, auth header
//! injection, JSON serialization, and `tracing` spans. Every service client
//! wraps an `HttpClient` instead of using `reqwest` directly.

use crate::core::auth::Auth;
use crate::core::error::ApiError;

/// Shared HTTP client wrapping `reqwest::Client` with auth and base URL.
///
/// Real implementation will hold a `reqwest::Client`, retry policy, and
/// rate limiter. This skeleton fixes the public surface so service clients
/// can compile against it.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    auth: Auth,
}

impl HttpClient {
    /// Construct a new client with a base URL and auth strategy.
    #[must_use]
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            base_url: base_url.into(),
            auth,
        }
    }

    /// Base URL this client targets.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Auth strategy.
    #[must_use]
    pub const fn auth(&self) -> &Auth {
        &self.auth
    }

    /// GET a path and decode JSON.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        _path: &str,
    ) -> Result<T, ApiError> {
        Err(ApiError::Internal(
            "HttpClient::get_json not yet implemented".into(),
        ))
    }

    /// POST a JSON body and decode the JSON response.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn post_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        _path: &str,
        _body: &B,
    ) -> Result<T, ApiError> {
        Err(ApiError::Internal(
            "HttpClient::post_json not yet implemented".into(),
        ))
    }
}
