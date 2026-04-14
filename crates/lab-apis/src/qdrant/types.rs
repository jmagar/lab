//! Qdrant request/response types.
//!
//! Modeled on the REST OpenAPI spec. Stub — grows as endpoints are wired.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a Qdrant collection name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CollectionName(pub String);

/// A collection entry returned by `GET /collections`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDescription {
    pub name: CollectionName,
}

/// Generic wrapper for Qdrant's `{ result: T, status: ..., time: ... }` envelope.
///
/// Used internally by the client to unwrap the outer result layer before returning.
#[derive(serde::Deserialize)]
pub(super) struct QdrantEnvelope<T> {
    pub result: T,
}

/// Metadata for a single Qdrant collection.
///
/// A partial view of the Qdrant collection info response. Fields are optional
/// because Qdrant may omit them for empty or initialising collections.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CollectionInfo {
    /// Collection status (`green`, `yellow`, `red`, or `grey`).
    pub status: String,
    /// Approximate number of vectors in the collection.
    pub vectors_count: Option<u64>,
    /// Number of indexed vectors.
    pub indexed_vectors_count: Option<u64>,
    /// Approximate number of points in the collection.
    pub points_count: Option<u64>,
    /// Number of segments in the collection.
    pub segments_count: Option<u64>,
}

/// Distance metric for a collection's vector space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Distance {
    Cosine,
    Euclid,
    Dot,
    Manhattan,
}

/// Vector-space configuration for a new collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorParams {
    pub size: u64,
    pub distance: Distance,
}

/// A point (vector + payload) stored in a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// Point id — Qdrant accepts numeric or UUID ids.
    pub id: PointId,
    pub vector: Vec<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// A point identifier — numeric or UUID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointId {
    Num(u64),
    Uuid(String),
}

/// A search hit returned from `POST /collections/{name}/points/search`.
#[derive(Debug, Clone, Deserialize)]
pub struct ScoredPoint {
    pub id: PointId,
    pub score: f32,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// Request / upsert types
// ---------------------------------------------------------------------------

/// Config for creating a new collection (`PUT /collections/{name}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollection {
    pub vectors: VectorParams,
}

/// A single point to upsert into a collection.
///
/// Qdrant accepts numeric or UUID ids; use `serde_json::Value` so callers
/// can pass either without the dispatch layer needing to distinguish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertPoint {
    /// Point identifier — numeric (`u64`) or UUID string.
    pub id: serde_json::Value,
    pub vector: Vec<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Request body sent to `POST /collections/{name}/points/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    pub limit: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub with_payload: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f32>,
}

/// Snapshot metadata returned by `POST /collections/{name}/snapshots`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnapshotInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Request body for `PUT /collections/{name}/index`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndex {
    pub field_name: String,
    pub field_schema: serde_json::Value,
}
