//! Sonarr client — not yet implemented.
//!
//! This module exists so the `sonarr` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the sonarr module.
pub const META: PluginMeta = PluginMeta {
    name: "sonarr",
    display_name: "Sonarr",
    description: "TV series management for the Servarr stack (placeholder — not yet implemented)",
    category: Category::Servarr,
    docs_url: "https://sonarr.tv/docs/api/",
    required_env: &[],
    optional_env: &[],
    default_port: Some(8989),
};
