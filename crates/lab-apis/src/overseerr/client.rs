//! Overseerr API client skeleton.

use crate::core::{Auth, HttpClient};

/// Async client for the Overseerr REST API.
pub struct OverseerrClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl OverseerrClient {
    /// Construct a new client targeting `base_url` with the given auth.
    #[must_use]
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            http: HttpClient::new(base_url, auth),
        }
    }
}
