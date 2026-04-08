//! Tailscale client — not yet implemented.
//!
//! This module exists so the `tailscale` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the tailscale module.
pub const META: PluginMeta = PluginMeta {
    name: "tailscale",
    display_name: "Tailscale",
    description: "WireGuard-based mesh VPN (placeholder — not yet implemented)",
    category: Category::Network,
    docs_url: "https://tailscale.com/api",
    required_env: &[],
    optional_env: &[],
    default_port: None,
};
