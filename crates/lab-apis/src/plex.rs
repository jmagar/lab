//! Plex client — not yet implemented.
//!
//! This module exists so the `plex` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `PlexClient` — media server methods.
pub mod client;

/// Plex request/response types (serde).
pub mod types;

/// `PlexError` (thiserror).
pub mod error;

pub use client::PlexClient;
pub use error::PlexError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the plex module.
pub const META: PluginMeta = PluginMeta {
    name: "plex",
    display_name: "Plex",
    description: "Plex media server (placeholder — not yet implemented)",
    category: Category::Media,
    docs_url: "https://www.plexopedia.com/plex-media-server/api/",
    required_env: &[],
    optional_env: &[],
    default_port: Some(32400),
};
