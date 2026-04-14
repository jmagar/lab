//! TEI request/response types.
//!
//! Grows as additional endpoints are wired.

use serde::{Deserialize, Serialize};

/// Request body for `POST /embed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankHit {
    pub index: u32,
    pub score: f32,
    #[serde(default)]
    pub text: Option<String>,
}

/// Request body for `POST /tokenize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeRequest {
    /// One or more input strings to tokenize.
    pub inputs: serde_json::Value,
    /// Whether to add special tokens (e.g. CLS/SEP).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_special_tokens: Option<bool>,
}

/// Request body for `POST /similarity`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityRequest {
    /// Array of `[sentence_a, sentence_b]` string pairs.
    pub inputs: Vec<[String; 2]>,
}

/// Request body for `POST /embed_sparse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseEmbedRequest {
    /// One or more input strings.
    pub inputs: serde_json::Value,
    /// Whether to truncate overlong inputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
}

/// A single sparse embedding token entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseEmbedToken {
    pub index: u32,
    pub value: f32,
}

/// Server metadata (from `GET /info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub model_id: String,
    pub model_sha: Option<String>,
    pub model_dtype: Option<String>,
    pub model_type: Option<serde_json::Value>,
    pub max_concurrent_requests: Option<u32>,
    pub max_input_length: Option<u32>,
    pub max_batch_tokens: Option<u32>,
    pub version: Option<String>,
}
