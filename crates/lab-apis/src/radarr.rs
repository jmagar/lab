//! Radarr movie-management client (API v3).
//!
//! Radarr is a fork of Sonarr focused on movies. It exposes ~100 REST
//! endpoints across a dozen resource groups. Rather than cram everything
//! into a single `client.rs`/`types.rs`, this module **splits by resource**:
//! each resource lives in its own sibling file under `client/` and `types/`,
//! and all the per-resource `impl RadarrClient` blocks compose into one
//! client struct.
//!
//! ```text
//! radarr/
//! ├── client.rs          # struct def, constructor, health, module decls
//! ├── client/
//! │   ├── movies.rs      # impl RadarrClient { async fn movie_* }
//! │   ├── queue.rs
//! │   ├── indexers.rs
//! │   ├── quality.rs
//! │   └── commands.rs
//! ├── types.rs           # re-exports from types/
//! ├── types/
//! │   ├── movie.rs
//! │   ├── queue.rs
//! │   ├── indexer.rs
//! │   ├── quality.rs
//! │   └── command.rs
//! └── error.rs
//! ```
//!
//! Spec: `docs/api-specs/radarr.openapi.json` (mirrored from
//! `github.com/Radarr/Radarr/blob/develop/src/Radarr.Api.V3/openapi.json`).
//!
//! Auth is header-token: `X-Api-Key: <key>`.

/// Public request/response types (serde). Split by resource.
pub mod types;

/// `RadarrError` (thiserror).
pub mod error;

/// `RadarrClient` struct + per-resource `impl` blocks.
pub mod client;

pub use client::RadarrClient;
pub use error::RadarrError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the radarr module.
pub const META: PluginMeta = PluginMeta {
    name: "radarr",
    display_name: "Radarr",
    description: "Movie collection manager for Usenet and BitTorrent",
    category: Category::Servarr,
    docs_url: "https://radarr.video/docs/api/",
    required_env: &[
        EnvVar {
            name: "RADARR_URL",
            description: "Base URL of the Radarr instance",
            example: "http://localhost:7878",
            secret: false,
        },
        EnvVar {
            name: "RADARR_API_KEY",
            description: "API key from Settings → General",
            example: "abc123def456...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(7878),
};
