//! ByteStash code-snippet client.
//!
//! ByteStash is a self-hosted snippet manager with JWT bearer auth and a small
//! REST surface: auth, snippets, shared snippets, categories, and admin users.

/// `ByteStashClient` — snippet management methods.
pub mod client;

/// Request/response types for the documented ByteStash payloads.
pub mod types;

/// `ByteStashError` (thiserror).
pub mod error;

pub use client::ByteStashClient;
pub use error::ByteStashError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};

/// Compile-time metadata for the bytestash module.
pub const META: PluginMeta = PluginMeta {
    name: "bytestash",
    display_name: "ByteStash",
    description: "Self-hosted code snippet manager",
    category: Category::Notes,
    docs_url: "https://github.com/bytestash/bytestash",
    required_env: &[
        EnvVar {
            name: "BYTESTASH_URL",
            description: "Base URL of the ByteStash instance",
            example: "http://localhost:5000",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "BYTESTASH_TOKEN",
            description: "JWT bearer token for the API",
            example: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(5000),
    supports_multi_instance: false,
};
