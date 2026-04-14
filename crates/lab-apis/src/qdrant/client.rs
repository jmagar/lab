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

    /// Create a collection — `PUT /collections/{name}`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure, decode failure, or if `name` is empty.
    pub async fn collection_create(
        &self,
        name: &str,
        body: &super::types::CreateCollection,
    ) -> Result<serde_json::Value, QdrantError> {
        if name.is_empty() {
            return Err(QdrantError::Api(crate::core::ApiError::Validation {
                field: "name".into(),
                message: "collection name must not be empty".into(),
            }));
        }
        let encoded = encode_path_segment(name);
        let path = format!("/collections/{encoded}");
        let val: serde_json::Value = self.http.put_json(&path, body).await?;
        Ok(val)
    }

    /// Delete a collection — `DELETE /collections/{name}`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or if `name` is empty.
    pub async fn collection_delete(&self, name: &str) -> Result<(), QdrantError> {
        if name.is_empty() {
            return Err(QdrantError::Api(crate::core::ApiError::Validation {
                field: "name".into(),
                message: "collection name must not be empty".into(),
            }));
        }
        let encoded = encode_path_segment(name);
        let path = format!("/collections/{encoded}");
        self.http.delete(&path).await?;
        Ok(())
    }

    /// Upsert a single chunk of points — `PUT /collections/{name}/points`.
    ///
    /// Internal method; callers should prefer [`Self::point_upsert_batched`] for
    /// large sets of points.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_upsert(
        &self,
        collection: &str,
        points: Vec<super::types::UpsertPoint>,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points");
        let body = serde_json::json!({ "points": points });
        let val: serde_json::Value = self.http.put_json(&path, &body).await?;
        Ok(val)
    }

    /// Upsert points with automatic chunking at 500 points per request.
    ///
    /// Large batches are split into chunks of 500 and sent sequentially.
    /// On failure the method returns immediately with context identifying which
    /// chunk failed — partial commits up to the failed chunk are already
    /// persisted (Qdrant upserts are idempotent, so retrying from the
    /// beginning is safe).
    ///
    /// # Errors
    /// Returns `QdrantError::Api` with chunk context on any chunk failure.
    pub async fn point_upsert_batched(
        &self,
        collection: &str,
        points: Vec<super::types::UpsertPoint>,
    ) -> Result<serde_json::Value, QdrantError> {
        const CHUNK_SIZE: usize = 500;
        let chunks: Vec<_> = points.chunks(CHUNK_SIZE).collect();
        let num_chunks = chunks.len();
        let mut last_result = serde_json::json!({ "status": "ok" });

        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_vec = chunk.to_vec();
            let start = i * CHUNK_SIZE;
            let end = (start + chunk_vec.len()).saturating_sub(1);
            last_result = self
                .point_upsert(collection, chunk_vec)
                .await
                .map_err(|e| {
                    QdrantError::Api(crate::core::ApiError::Internal(format!(
                        "failed on chunk {} of {} (points {}-{}): {}",
                        i + 1,
                        num_chunks,
                        start,
                        end,
                        e,
                    )))
                })?;
        }

        Ok(last_result)
    }

    /// Search for nearest neighbours — `POST /collections/{name}/points/search`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_search(
        &self,
        collection: &str,
        req: &super::types::SearchRequest,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points/search");
        let val: serde_json::Value = self.http.post_json(&path, req).await?;
        Ok(val)
    }

    /// Universal query endpoint — `POST /collections/{name}/points/query`.
    ///
    /// Accepts a raw `serde_json::Value` body so callers can express any
    /// query shape supported by the Qdrant spec without SDK coupling.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_query(
        &self,
        collection: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points/query");
        let val: serde_json::Value = self.http.post_json(&path, body).await?;
        Ok(val)
    }

    /// Scroll through points — `POST /collections/{name}/points/scroll`.
    ///
    /// Accepts a raw `serde_json::Value` body for full spec compatibility.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_scroll(
        &self,
        collection: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points/scroll");
        let val: serde_json::Value = self.http.post_json(&path, body).await?;
        Ok(val)
    }

    /// Count points matching an optional filter — `POST /collections/{name}/points/count`.
    ///
    /// Accepts a raw `serde_json::Value` body for full spec compatibility.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_count(
        &self,
        collection: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points/count");
        let val: serde_json::Value = self.http.post_json(&path, body).await?;
        Ok(val)
    }

    /// Delete points by ids or filter — `POST /collections/{name}/points/delete`.
    ///
    /// Accepts a raw `serde_json::Value` body (e.g. `{"points": [1, 2]}` or
    /// `{"filter": {...}}`).
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn point_delete(
        &self,
        collection: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/points/delete");
        let val: serde_json::Value = self.http.post_json(&path, body).await?;
        Ok(val)
    }

    /// Create a snapshot — `POST /collections/{name}/snapshots`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn snapshot_create(
        &self,
        collection: &str,
    ) -> Result<super::types::SnapshotInfo, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/snapshots");
        let body = serde_json::json!({});
        let envelope: super::types::QdrantEnvelope<super::types::SnapshotInfo> =
            self.http.post_json(&path, &body).await?;
        Ok(envelope.result)
    }

    /// Create a payload index — `PUT /collections/{name}/index`.
    ///
    /// # Errors
    /// Returns `QdrantError::Api` on HTTP failure or decode failure.
    pub async fn index_create(
        &self,
        collection: &str,
        req: &super::types::CreateIndex,
    ) -> Result<serde_json::Value, QdrantError> {
        let encoded = encode_path_segment(collection);
        let path = format!("/collections/{encoded}/index");
        let val: serde_json::Value = self.http.put_json(&path, req).await?;
        Ok(val)
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
