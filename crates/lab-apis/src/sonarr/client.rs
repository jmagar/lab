//! `SonarrClient` — TV series management methods.
//!
//! TODO: implement series/episode/queue management from the Sonarr API v3.

use crate::core::{Auth, HttpClient};

use super::error::SonarrError;

/// Client for a Sonarr TV series manager instance.
pub struct SonarrClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl SonarrClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Sonarr uses header-token auth: pass
    /// `Auth::ApiKey { header: "X-Api-Key".into(), key: api_key }`.
    ///
    /// # Errors
    /// Returns [`SonarrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, SonarrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), SonarrError> {
        // TODO: GET /api/v3/system/status
        Ok(())
    }
}
