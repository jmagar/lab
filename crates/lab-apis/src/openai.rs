//! OpenAI API client.
//!
//! Wraps the OpenAI REST API (chat completions, embeddings, models, images,
//! audio, moderation). Auth is `Authorization: Bearer <api-key>`.
//!
//! Spec: `docs/api-specs/openai.openapi.yaml` (mirrored from
//! `github.com/openai/openai-openapi`, `manual_spec` branch).
//!
//! Note: this client targets the public `api.openai.com` endpoint by default,
//! but any OpenAI-compatible server (Ollama, vLLM, LiteLLM, etc.) works by
//! pointing `base_url` at it.

/// Public request/response types (serde).
pub mod types;

/// `OpenAiError` (thiserror).
pub mod error;

/// `OpenAiClient` — chat, embeddings, models, images, audio, moderations.
pub mod client;

pub use client::OpenAiClient;
pub use error::OpenAiError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the openai module.
pub const META: PluginMeta = PluginMeta {
    name: "openai",
    display_name: "OpenAI",
    description: "OpenAI API (chat, embeddings, models, images, audio)",
    category: Category::Ai,
    docs_url: "https://platform.openai.com/docs/api-reference",
    required_env: &[EnvVar {
        name: "OPENAI_API_KEY",
        description: "OpenAI API key",
        example: "sk-proj-...",
        secret: true,
    }],
    optional_env: &[
        EnvVar {
            name: "OPENAI_URL",
            description: "Override base URL (for OpenAI-compatible servers)",
            example: "http://localhost:11434/v1",
            secret: false,
        },
        EnvVar {
            name: "OPENAI_ORG_ID",
            description: "Organization id for billing/quota",
            example: "org-...",
            secret: false,
        },
    ],
    default_port: None,
};
