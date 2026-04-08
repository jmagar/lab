//! Tautulli client — not yet implemented.
//!
//! This module exists so the `tautulli` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the tautulli module.
pub const META: PluginMeta = PluginMeta {
    name: "tautulli",
    display_name: "Tautulli",
    description: "Plex analytics and monitoring (placeholder — not yet implemented)",
    category: Category::Media,
    docs_url: "https://github.com/Tautulli/Tautulli/wiki/Tautulli-API-Reference",
    required_env: &[],
    optional_env: &[],
    default_port: Some(8181),
};
