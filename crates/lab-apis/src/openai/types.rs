//! OpenAI request/response types.
//!
//! Modeled on the official OpenAI OpenAPI spec. Chat, embeddings, and models
//! are the primary surfaces; images, audio, and moderation will be added as needed.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around an `OpenAI` model identifier (e.g. "gpt-4o-mini").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(pub String);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Chat completions ─────────────────────────────────────────────────────────

/// One entry in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Chat message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Request body for `POST /v1/chat/completions`.
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: ModelId,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Response from `POST /v1/chat/completions` (non-streaming).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A single choice in a chat completion response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Token usage stats for a completion.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── Embeddings ───────────────────────────────────────────────────────────────

/// Request body for `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingsRequest {
    pub model: ModelId,
    pub input: EmbeddingsInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// Input can be a single string or an array of strings.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Single(String),
    Batch(Vec<String>),
}

/// Response from `POST /v1/embeddings`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingsResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A single embedding vector.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Embedding {
    pub index: u32,
    pub embedding: Vec<f32>,
    pub object: String,
}

// ── Models ───────────────────────────────────────────────────────────────────

/// Response from `GET /v1/models`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

/// A single model entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}
