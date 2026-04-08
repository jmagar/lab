//! Paperless-ngx client — not yet implemented.
//!
//! This module exists so the `paperless` feature compiles. The real client,
//! types, and MCP dispatch are deferred to a per-service plan.

/// `PaperlessClient` — document management methods.
pub mod client;

/// Paperless request/response types (serde).
pub mod types;

/// `PaperlessError` (thiserror).
pub mod error;

pub use client::PaperlessClient;
pub use error::PaperlessError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the paperless module.
pub const META: PluginMeta = PluginMeta {
    name: "paperless",
    display_name: "Paperless-ngx",
    description: "Document management system (placeholder — not yet implemented)",
    category: Category::Documents,
    docs_url: "https://docs.paperless-ngx.com/api/",
    required_env: &[],
    optional_env: &[],
    default_port: Some(8000),
};
