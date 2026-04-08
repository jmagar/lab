//! `TeiClient` — async methods against a Hugging Face TEI server.
//!
//! Stub. Endpoints land incrementally from `docs/api-specs/tei.openapi.json`.

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
    #[must_use]
    pub fn new(base_url: &str, auth: Auth) -> Self {
        Self {
            http: HttpClient::new(base_url, auth),
        }
    }

    /// Health check — `GET /health`.
    ///
    /// # Errors
    /// Returns `TeiError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), TeiError> {
        // TODO: GET /health
        Ok(())
    }
}
