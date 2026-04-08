//! Prowlarr client — not yet implemented.
//!
//! This module exists so the `prowlarr` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `ProwlarrClient` — indexer manager methods.
pub mod client;

/// Prowlarr request/response types (serde).
pub mod types;

/// `ProwlarrError` (thiserror).
pub mod error;

pub use client::ProwlarrClient;
pub use error::ProwlarrError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the prowlarr module.
pub const META: PluginMeta = PluginMeta {
    name: "prowlarr",
    display_name: "Prowlarr",
    description: "Indexer manager for the Servarr stack (placeholder — not yet implemented)",
    category: Category::Indexer,
    docs_url: "https://prowlarr.com/docs/api/",
    required_env: &[],
    optional_env: &[],
    default_port: Some(9696),
};
