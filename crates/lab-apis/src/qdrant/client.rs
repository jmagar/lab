//! `QdrantClient` — async methods against a Qdrant REST API.
//!
//! Endpoints land incrementally from `docs/upstream-api/qdrant.openapi.json`.

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
    ///
    /// # Errors
    /// Returns [`QdrantError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, QdrantError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check — `GET /healthz`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), QdrantError> {
        self.http.get_void("/healthz").await?;
        Ok(())
    }

    /// List collection names.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn collections_list(
        &self,
    ) -> Result<Vec<super::types::CollectionDescription>, QdrantError> {
        #[derive(serde::Deserialize)]
        struct CollectionList {
            collections: Vec<super::types::CollectionDescription>,
        }

        let envelope: super::types::QdrantEnvelope<CollectionList> =
            self.http.get_json("/collections").await?;
        Ok(envelope.result.collections)
    }

    /// Fetch raw collection metadata.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    // TODO: type the return value once callers' field needs are known.
    pub async fn collection_get(&self, name: &str) -> Result<serde_json::Value, QdrantError> {
        let path = format!("/collections/{name}");
        let envelope: super::types::QdrantEnvelope<serde_json::Value> =
            self.http.get_json(&path).await?;
        Ok(envelope.result)
    }
}
