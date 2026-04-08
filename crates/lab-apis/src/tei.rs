//! Hugging Face Text Embeddings Inference (TEI) client.
//!
//! TEI is a high-performance inference server for embedding, reranker, and
//! sequence-classification models. It exposes a small REST surface that is
//! partially OpenAI-compatible (`/embed` + `/v1/embeddings`).
//!
//! Spec: `docs/api-specs/tei.openapi.json` (mirrored from
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

use crate::core::plugin::{Category, EnvVar, PluginMeta};

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
    }],
    optional_env: &[EnvVar {
        name: "TEI_API_KEY",
        description: "Bearer token (only when launched with --api-key)",
        example: "abc123...",
        secret: true,
    }],
    default_port: Some(80),
};
