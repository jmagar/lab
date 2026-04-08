//! ByteStash client — not yet implemented.
//!
//! This module exists so the `bytestash` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the bytestash module.
pub const META: PluginMeta = PluginMeta {
    name: "bytestash",
    display_name: "ByteStash",
    description: "Self-hosted code snippet manager (placeholder — not yet implemented)",
    category: Category::Notes,
    docs_url: "https://github.com/bytestash/bytestash",
    required_env: &[],
    optional_env: &[],
    default_port: Some(5000),
};
