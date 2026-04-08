//! qBittorrent client — not yet implemented.
//!
//! This module exists so the `qbittorrent` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the qbittorrent module.
pub const META: PluginMeta = PluginMeta {
    name: "qbittorrent",
    display_name: "qBittorrent",
    description: "Torrent download client (placeholder — not yet implemented)",
    category: Category::Download,
    docs_url: "https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)",
    required_env: &[],
    optional_env: &[],
    default_port: Some(8080),
};
