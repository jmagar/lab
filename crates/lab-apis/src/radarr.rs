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

use std::time::Instant;

use crate::core::{ApiError, ServiceClient, ServiceStatus};

impl ServiceClient for RadarrClient {
    fn name(&self) -> &'static str {
        "radarr"
    }

    fn service_type(&self) -> &'static str {
        "servarr"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.system_status().await {
            Ok(status) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: Some(status.version),
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(RadarrError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(RadarrError::Api(ApiError::Network(e))) => Ok(ServiceStatus::unreachable(e)),
            Err(RadarrError::Api(e)) => Ok(ServiceStatus::degraded(e.to_string())),
            Err(e) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
