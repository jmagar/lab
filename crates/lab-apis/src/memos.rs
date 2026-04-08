//! Memos client — not yet implemented.
//!
//! This module exists so the `memos` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the memos module.
pub const META: PluginMeta = PluginMeta {
    name: "memos",
    display_name: "Memos",
    description: "Lightweight self-hosted memo hub (placeholder — not yet implemented)",
    category: Category::Notes,
    docs_url: "https://www.usememos.com/docs/api-documentation",
    required_env: &[],
    optional_env: &[],
    default_port: Some(5230),
};
