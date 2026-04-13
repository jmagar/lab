//! `TeiClient` — async methods against a Hugging Face TEI server.
//!
//! Endpoints land incrementally from `docs/upstream-api/tei.openapi.json`.

use crate::core::{Auth, HttpClient};

use super::error::TeiError;

/// Client for a Text Embeddings Inference server.
pub struct TeiClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl TeiClient {
    /// Build a client against `base_url`.
    ///
    /// TEI uses `Authorization: Bearer <token>` when launched with
    /// `--api-key`. For unauthenticated local instances, pass `Auth::None`.
    ///
    /// # Errors
    /// Returns [`TeiError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, TeiError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check — `GET /health`.
    ///
    /// # Errors
    /// Returns `TeiError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), TeiError> {
        self.http.get_void("/health").await?;
        Ok(())
    }

    /// Fetch model and server metadata.
    ///
    /// # Errors
    /// Returns `TeiError::Api` on HTTP failure or decode failure.
    pub async fn model_info(&self) -> Result<super::types::Info, TeiError> {
        Ok(self.http.get_json("/info").await?)
    }

    /// Request embeddings for one or more input strings.
    ///
    /// # Errors
    /// Returns `TeiError::Api` on HTTP failure or decode failure.
    pub async fn embed(
        &self,
        request: &super::types::EmbedRequest,
    ) -> Result<Vec<Vec<f32>>, TeiError> {
        Ok(self.http.post_json("/embed", request).await?)
    }
}
