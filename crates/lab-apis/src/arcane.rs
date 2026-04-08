//! Arcane client — not yet implemented.
//!
//! This module exists so the `arcane` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `ArcaneClient` — Docker management methods.
pub mod client;

/// Arcane request/response types (serde).
pub mod types;

/// `ArcaneError` (thiserror).
pub mod error;

pub use client::ArcaneClient;
pub use error::ArcaneError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the arcane module.
pub const META: PluginMeta = PluginMeta {
    name: "arcane",
    display_name: "Arcane",
    description: "Docker management UI (placeholder — not yet implemented)",
    category: Category::Network,
    docs_url: "https://github.com/arcane-app/arcane",
    required_env: &[],
    optional_env: &[],
    default_port: Some(3000),
};
