//! `ProwlarrClient` — indexer manager methods.
//!
//! TODO: implement indexer management, search, and history from the Prowlarr API.

use crate::core::{Auth, HttpClient};

use super::error::ProwlarrError;

/// Client for a Prowlarr indexer manager instance.
pub struct ProwlarrClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl ProwlarrClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Prowlarr uses header-token auth: pass
    /// `Auth::ApiKey { header: "X-Api-Key".into(), key: api_key }`.
    ///
    /// # Errors
    /// Returns [`ProwlarrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ProwlarrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), ProwlarrError> {
        // TODO: GET /api/v1/system/status
        Ok(())
    }
}
