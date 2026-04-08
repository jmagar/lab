//! Linkding client — not yet implemented.
//!
//! This module exists so the `linkding` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `LinkdingClient` — bookmark management methods.
pub mod client;

/// Linkding request/response types (serde).
pub mod types;

/// `LinkdingError` (thiserror).
pub mod error;

pub use client::LinkdingClient;
pub use error::LinkdingError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the linkding module.
pub const META: PluginMeta = PluginMeta {
    name: "linkding",
    display_name: "Linkding",
    description: "Self-hosted bookmark manager (placeholder — not yet implemented)",
    category: Category::Notes,
    docs_url: "https://github.com/sissbruecker/linkding/blob/master/docs/API.md",
    required_env: &[],
    optional_env: &[],
    default_port: Some(9090),
};
