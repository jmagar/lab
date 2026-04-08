//! `LinkdingClient` — bookmark management methods.
//!
//! TODO: implement bookmark CRUD, tag management, and search from the Linkding REST API.

use crate::core::{Auth, HttpClient};

use super::error::LinkdingError;

/// Client for a Linkding bookmark manager instance.
pub struct LinkdingClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl LinkdingClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Linkding uses token auth: pass `Auth::Token { token: api_token }`.
    ///
    /// # Errors
    /// Returns [`LinkdingError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, LinkdingError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `LinkdingError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), LinkdingError> {
        // TODO: GET /api/bookmarks/ with page_size=1
        Ok(())
    }
}
