//! `SabnzbdClient` — Usenet download management methods.
//!
//! TODO: implement queue, history, config, and NZB-add operations from the `SABnzbd` API.

use crate::core::{Auth, HttpClient};

use super::error::SabnzbdError;

/// Client for a `SABnzbd` instance.
pub struct SabnzbdClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl SabnzbdClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// `SABnzbd` uses API key auth: pass `Auth::ApiKey { header: "...", key: api_key }`.
    ///
    /// # Errors
    /// Returns [`SabnzbdError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, SabnzbdError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), SabnzbdError> {
        // TODO: GET /api?mode=version
        Ok(())
    }
}
