//! UniFi client — not yet implemented.
//!
//! This module exists so the `unifi` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the unifi module.
pub const META: PluginMeta = PluginMeta {
    name: "unifi",
    display_name: "UniFi",
    description: "UniFi Network Application local API (placeholder — not yet implemented)",
    category: Category::Network,
    docs_url: "https://ubntwiki.com/products/software/unifi-controller/api",
    required_env: &[],
    optional_env: &[],
    default_port: Some(443),
};
