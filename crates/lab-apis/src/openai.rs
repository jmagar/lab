//! OpenAI API client.
//!
//! Wraps the OpenAI REST API (chat completions, embeddings, models).
//! Auth is `Authorization: Bearer <api-key>`.
//!
//! Any OpenAI-compatible server (Ollama, vLLM, LiteLLM, etc.) works by
//! pointing `base_url` at it.

/// Public request/response types (serde).
pub mod types;

/// `OpenAiError` (thiserror).
pub mod error;

/// `OpenAiClient` — chat, embeddings, models.
pub mod client;

pub use client::OpenAiClient;
pub use error::OpenAiError;

use std::time::Instant;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

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

impl ServiceClient for OpenAiClient {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn service_type(&self) -> &'static str {
        "ai"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.list_models().await {
            Ok(_) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                #[allow(clippy::cast_possible_truncation)]
                latency_ms: start.elapsed().as_millis() as u64,
                message: None,
            }),
            Err(OpenAiError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                #[allow(clippy::cast_possible_truncation)]
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some("authentication failed".to_string()),
            }),
            Err(OpenAiError::Api(ApiError::Network(e))) => Ok(ServiceStatus::unreachable(e)),
            Err(e) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
