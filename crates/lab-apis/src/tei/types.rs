//! TEI request/response types.
//!
//! Stub — grows as endpoints are wired.

use serde::{Deserialize, Serialize};

/// Request body for `POST /embed`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbedRequest {
    /// One or more input strings to embed.
    pub inputs: Vec<String>,
    /// Whether to L2-normalize the returned vectors (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize: Option<bool>,
    /// Whether to truncate inputs that exceed the model's max length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
}

/// Request body for `POST /rerank`.
#[derive(Debug, Clone, Serialize)]
pub struct RerankRequest {
    pub query: String,
    pub texts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_scores: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
}

/// A single rerank result.
#[derive(Debug, Clone, Deserialize)]
pub struct RerankHit {
    pub index: u32,
    pub score: f32,
    #[serde(default)]
    pub text: Option<String>,
}

/// Server metadata (from `GET /info`).
#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub model_id: String,
    pub model_sha: Option<String>,
    pub model_dtype: Option<String>,
    pub model_type: Option<String>,
    pub max_concurrent_requests: Option<u32>,
    pub max_input_length: Option<u32>,
    pub max_batch_tokens: Option<u32>,
    pub version: Option<String>,
}
