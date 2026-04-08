//! `PlexClient` — media server methods.
//!
//! TODO: implement library browsing, playback control, and session management from the Plex HTTP API.

use crate::core::{Auth, HttpClient};

use super::error::PlexError;

/// Client for a Plex Media Server instance.
pub struct PlexClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl PlexClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Plex uses token auth: pass `Auth::Token { token: x_plex_token }`.
    ///
    /// # Errors
    /// Returns [`PlexError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, PlexError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), PlexError> {
        // TODO: GET / (identity endpoint)
        Ok(())
    }
}
