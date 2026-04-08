//! SABnzbd client — not yet implemented.
//!
//! This module exists so the `sabnzbd` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `SabnzbdClient` — Usenet download management methods.
pub mod client;

/// SABnzbd request/response types (serde).
pub mod types;

/// `SabnzbdError` (thiserror).
pub mod error;

pub use client::SabnzbdClient;
pub use error::SabnzbdError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the sabnzbd module.
pub const META: PluginMeta = PluginMeta {
    name: "sabnzbd",
    display_name: "SABnzbd",
    description: "Usenet download client (placeholder — not yet implemented)",
    category: Category::Download,
    docs_url: "https://sabnzbd.org/wiki/configuration/4.0/api",
    required_env: &[],
    optional_env: &[],
    default_port: Some(8080),
};
