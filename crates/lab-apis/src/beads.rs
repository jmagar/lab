//! Beads — local task and issue tracker backed by Dolt MySQL.
//!
//! Connects to a local Dolt server (MySQL wire protocol) running on a port
//! discovered at runtime from `.beads/dolt-server.port`. The default database
//! is `lab` with user `root` and no password.
//!
//! Read-only v1 — list and get issues. Writes (create, update, close, comment)
//! are deferred to a follow-up bead.

/// `BeadsClient` — Dolt MySQL access for the beads schema.
pub mod client;

/// Beads error taxonomy.
pub mod error;

/// Request/response types.
pub mod types;

pub use client::BeadsClient;
pub use error::BeadsError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the beads module.
pub const META: PluginMeta = PluginMeta {
    name: "beads",
    display_name: "Beads",
    description: "Local task and issue tracker backed by Dolt MySQL",
    category: Category::Bootstrap,
    docs_url: "https://github.com/steveyegge/beads",
    required_env: &[],
    optional_env: &[],
    default_port: None,
    supports_multi_instance: false,
};
