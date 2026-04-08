//! Overseerr API client skeleton.

use crate::core::{Auth, HttpClient};

use super::error::OverseerrError;

/// Async client for the Overseerr REST API.
pub struct OverseerrClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl OverseerrClient {
    /// Construct a new client targeting `base_url` with the given auth.
    ///
    /// # Errors
    /// Returns [`OverseerrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Result<Self, OverseerrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }
}
