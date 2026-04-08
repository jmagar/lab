//! `ArcaneClient` — Docker management methods.
//!
//! TODO: implement container/image/volume management from the Arcane REST API.

use crate::core::{Auth, HttpClient};

use super::error::ArcaneError;

/// Client for an Arcane Docker management UI instance.
pub struct ArcaneClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl ArcaneClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// # Errors
    /// Returns [`ArcaneError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ArcaneError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), ArcaneError> {
        // TODO: GET /api/health or equivalent
        Ok(())
    }
}
