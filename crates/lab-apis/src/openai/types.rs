//! OpenAI request/response types.
//!
//! Modeled on the official OpenAI OpenAPI spec. Only the most-used
//! endpoints are wired here for now — chat, embeddings, models. Images,
//! audio, and moderation types will be added as needed.
//!
//! This is a stub — real types land as endpoints are implemented.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around an `OpenAI` model identifier (e.g. "gpt-4o-mini").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(pub String);

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

/// Request body for `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingsRequest {
    pub model: ModelId,
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

/// A single embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    pub index: u32,
    pub embedding: Vec<f32>,
}
