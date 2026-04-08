//! Qdrant vector database client.
//!
//! Wraps the Qdrant REST API — collections, points, search, snapshots,
//! cluster operations. Auth is `api-key` header (optional; default bind is
//! unauthenticated).
//!
//! Spec: `docs/api-specs/qdrant.openapi.json` (mirrored from
//! `github.com/qdrant/qdrant/blob/master/docs/redoc/master/openapi.json`).
//!
//! Note: Qdrant also exposes a gRPC API — this client speaks REST only. Use
//! the official `qdrant-client` crate if you need gRPC.

/// Public request/response types (serde).
pub mod types;

/// `QdrantError` (thiserror).
pub mod error;

/// `QdrantClient` — collections, points, search, snapshots.
pub mod client;

pub use client::QdrantClient;
pub use error::QdrantError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the qdrant module.
pub const META: PluginMeta = PluginMeta {
    name: "qdrant",
    display_name: "Qdrant",
    description: "Vector database for similarity search and RAG",
    category: Category::Ai,
    docs_url: "https://api.qdrant.tech/",
    required_env: &[EnvVar {
        name: "QDRANT_URL",
        description: "Base URL of the Qdrant server",
        example: "http://localhost:6333",
        secret: false,
    }],
    optional_env: &[EnvVar {
        name: "QDRANT_API_KEY",
        description: "API key (only if Qdrant is auth-protected)",
        example: "abc123...",
        secret: true,
    }],
    default_port: Some(6333),
};
