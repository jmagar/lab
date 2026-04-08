//! `GotifyClient` — async methods against a Gotify server.
//!
//! Stub: only the constructor and the shared `HttpClient` plumbing are wired
//! up. Per-endpoint methods (`send`, `messages`, `applications`, `clients`)
//! will be implemented from `docs/api-specs/gotify.openapi.json`.

use crate::core::{Auth, HttpClient};

use super::error::GotifyError;

/// Client for a Gotify server.
pub struct GotifyClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl GotifyClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Gotify uses header-token auth: pass
    /// `Auth::ApiKey { header: "X-Gotify-Key".into(), key: token }`.
    ///
    /// # Errors
    /// Returns [`GotifyError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, GotifyError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), GotifyError> {
        // TODO: GET /health
        Ok(())
    }
}
