//! `OpenAiClient` — async methods against OpenAI (or any OpenAI-compatible server).
//!
//! Stub. Endpoints land incrementally from `docs/api-specs/openai.openapi.yaml`.

use crate::core::{Auth, HttpClient};

use super::error::OpenAiError;

/// Client for the `OpenAI` API.
pub struct OpenAiClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl OpenAiClient {
    /// Build a client. Typical usage:
    ///
    /// ```no_run
    /// use lab_apis::core::Auth;
    /// use lab_apis::openai::OpenAiClient;
    /// let c = OpenAiClient::new("https://api.openai.com", Auth::Bearer { token: "sk-...".into() }).unwrap();
    /// ```
    ///
    /// # Errors
    /// Returns [`OpenAiError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, OpenAiError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Ping the models endpoint as a cheap health probe.
    ///
    /// # Errors
    /// Returns `OpenAiError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), OpenAiError> {
        // TODO: GET /v1/models
        Ok(())
    }
}
