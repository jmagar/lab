//! `QdrantClient` — async methods against a Qdrant REST API.
//!
//! Endpoints land incrementally from `docs/upstream-api/qdrant.openapi.json`.

use std::fmt::Write as _;

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

    /// Fetch collection metadata.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure, decode failure, or if `name` is empty.
    pub async fn collection_get(
        &self,
        name: &str,
    ) -> Result<super::types::CollectionInfo, QdrantError> {
        if name.is_empty() {
            return Err(QdrantError::Api(crate::core::ApiError::Validation {
                field: "name".into(),
                message: "collection name must not be empty".into(),
            }));
        }
        let encoded = encode_path_segment(name);
        let path = format!("/collections/{encoded}");
        let envelope: super::types::QdrantEnvelope<super::types::CollectionInfo> =
            self.http.get_json(&path).await?;
        Ok(envelope.result)
    }
}

/// Percent-encode a URL path segment (RFC 3986 unreserved chars pass through).
fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            for byte in encoded.bytes() {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}
