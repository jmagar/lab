//! Qdrant request/response types.
//!
//! Modeled on the REST OpenAPI spec. Stub — grows as endpoints are wired.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a Qdrant collection name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CollectionName(pub String);

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
