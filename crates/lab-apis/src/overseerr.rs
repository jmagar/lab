//! Overseerr media-request management client.
//!
//! Overseerr is the request/approval frontend that sits in front of Sonarr,
//! Radarr, and Plex. It exposes a REST API documented at
//! `api.overseerr.dev` (OpenAPI spec mirrored under `docs/api-specs/`).
//!
//! Auth is header-token: `X-Api-Key: <key>` (same shape as Servarr).

/// Public request/response types (serde).
pub mod types;

/// `OverseerrError` (thiserror).
pub mod error;

/// `OverseerrClient` struct + operation impls.
pub mod client;

pub use client::OverseerrClient;
pub use error::OverseerrError;

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the overseerr module.
pub const META: PluginMeta = PluginMeta {
    name: "overseerr",
    display_name: "Overseerr",
    description: "Request and approval frontend for Plex, Sonarr, and Radarr",
    category: Category::Media,
    docs_url: "https://api.overseerr.dev/",
    required_env: &[],
    optional_env: &[],
    default_port: Some(5055),
};
