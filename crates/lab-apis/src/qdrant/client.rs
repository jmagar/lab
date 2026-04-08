//! `QdrantClient` — async methods against a Qdrant REST API.
//!
//! Stub. Endpoints land incrementally from `docs/api-specs/qdrant.openapi.json`.

use crate::core::{Auth, HttpClient};

use super::error::QdrantError;

/// Client for a Qdrant vector database.
pub struct QdrantClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl QdrantClient {
    /// Build a client against `base_url`.
    ///
    /// Qdrant uses an `api-key` header when auth is enabled:
    /// `Auth::ApiKey { header: "api-key".into(), key: <key> }`. Pass
    /// `Auth::None` for an unauthenticated local instance.
    #[must_use]
    pub fn new(base_url: &str, auth: Auth) -> Self {
        Self {
            http: HttpClient::new(base_url, auth),
        }
    }

    /// Health check — `GET /healthz`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), QdrantError> {
        // TODO: GET /healthz
        Ok(())
    }
}
