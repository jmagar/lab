//! `AppriseClient` — async methods against an apprise-api server.
//!
//! Stub. Endpoints land incrementally from `docs/api-specs/apprise.md`.

use crate::core::{Auth, HttpClient};

use super::error::AppriseError;

/// Client for an apprise-api notification dispatcher.
pub struct AppriseClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl AppriseClient {
    /// Build a client against `base_url`.
    ///
    /// apprise-api is typically unauthenticated (pass `Auth::None`) but can
    /// be fronted by a reverse proxy that injects basic auth or a bearer
    /// token — use `Auth::Basic { .. }` or `Auth::Bearer { .. }` accordingly.
    ///
    /// # Errors
    /// Returns [`AppriseError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, AppriseError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check — `GET /status`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), AppriseError> {
        // TODO: GET /status
        Ok(())
    }
}
