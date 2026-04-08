//! Unraid client — not yet implemented.
//!
//! This module exists so the `unraid` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the unraid module.
pub const META: PluginMeta = PluginMeta {
    name: "unraid",
    display_name: "Unraid",
    description: "Unraid server GraphQL API (placeholder — not yet implemented)",
    category: Category::Network,
    docs_url: "https://docs.unraid.net/",
    required_env: &[],
    optional_env: &[],
    default_port: None,
};
