//! `PaperlessClient` — document management methods.
//!
//! TODO: implement document CRUD, correspondent/tag/type management from the Paperless-ngx REST API.

use crate::core::{Auth, HttpClient};

use super::error::PaperlessError;

/// Client for a Paperless-ngx instance.
pub struct PaperlessClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl PaperlessClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Paperless-ngx uses token auth: pass `Auth::Token { token: api_token }`.
    ///
    /// # Errors
    /// Returns [`PaperlessError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, PaperlessError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `PaperlessError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), PaperlessError> {
        // TODO: GET /api/ — wire real probe before marking healthy
        Err(PaperlessError::Api(crate::core::ApiError::Internal(
            "health check not yet implemented".into(),
        )))
    }
}
