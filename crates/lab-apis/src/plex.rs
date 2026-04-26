//! Plex Media Server client.
//!
//! Supports library browsing, media search, playback session management,
//! and playlist operations via the Plex HTTP API.
//!
//! Auth: `X-Plex-Token` header.

/// `PlexClient` — media server methods.
pub mod client;

/// Plex request/response types (serde).
pub mod types;

/// `PlexError` (thiserror).
pub mod error;

pub use client::PlexClient;
pub use error::PlexError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the plex module.
pub const META: PluginMeta = PluginMeta {
    name: "plex",
    display_name: "Plex",
    description: "Plex Media Server — library browsing, session management, and playlists",
    category: Category::Media,
    docs_url: "https://www.plexopedia.com/plex-media-server/api/",
    required_env: &[
        EnvVar {
            name: "PLEX_URL",
            description: "Base URL of the Plex Media Server instance",
            example: "http://localhost:32400",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "PLEX_TOKEN",
            description: "Plex authentication token (X-Plex-Token)",
            example: "xxxxxxxxxxxxxxxxxxxx",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(32400),
    supports_multi_instance: false,
};

impl ServiceClient for PlexClient {
    fn name(&self) -> &'static str {
        "plex"
    }

    fn service_type(&self) -> &'static str {
        "media"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.probe().await {
            Ok(()) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(PlexError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(PlexError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(PlexError::Api(e)) => Err(e),
        }
    }
}
