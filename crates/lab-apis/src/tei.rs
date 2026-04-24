//! Hugging Face Text Embeddings Inference (TEI) client.
//!
//! TEI is a high-performance inference server for embedding, reranker, and
//! sequence-classification models. It exposes a small REST surface that is
//! partially OpenAI-compatible (`/embed` + `/v1/embeddings`).
//!
//! Spec: `docs/upstream-api/tei.openapi.json` (mirrored from
//! `github.com/huggingface/text-embeddings-inference/blob/main/docs/openapi.json`).
//!
//! Auth is either `Authorization: Bearer <token>` (when the server is
//! launched with `--api-key`) or none for local instances.

/// Public request/response types (serde).
pub mod types;

/// `TeiError` (thiserror).
pub mod error;

/// `TeiClient` — embed, rerank, predict, tokenize.
pub mod client;

pub use client::TeiClient;
pub use error::TeiError;

use std::time::{Duration, Instant};

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the tei module.
pub const META: PluginMeta = PluginMeta {
    name: "tei",
    display_name: "HF Text Embeddings Inference",
    description: "Hugging Face TEI server — embeddings, rerankers, sequence classification",
    category: Category::Ai,
    docs_url: "https://huggingface.github.io/text-embeddings-inference/",
    required_env: &[EnvVar {
        name: "TEI_URL",
        description: "Base URL of the TEI server",
        example: "http://localhost:8080",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[EnvVar {
        name: "TEI_API_KEY",
        description: "Bearer token (only when launched with --api-key)",
        example: "abc123...",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: Some(80),
    supports_multi_instance: false,
};

impl ServiceClient for TeiClient {
    fn name(&self) -> &'static str {
        "tei"
    }

    fn service_type(&self) -> &'static str {
        "ai"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        let probe = tokio::time::timeout(Duration::from_secs(5), self.model_info()).await;
        match probe {
            Err(_elapsed) => Ok(ServiceStatus::unreachable("health check timed out")),
            Ok(Ok(info)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: info.version,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some(format!("model={}", info.model_id)),
            }),
            Ok(Err(TeiError::Api(ApiError::Auth))) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("authentication failed".into()),
            }),
            Ok(Err(TeiError::Api(ApiError::Network(err)))) => Ok(ServiceStatus::unreachable(err)),
            Ok(Err(TeiError::Api(err))) => Ok(ServiceStatus::degraded(err.to_string())),
        }
    }
}
