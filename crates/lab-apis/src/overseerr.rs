//! Overseerr media-request management client.
//!
//! Overseerr is the request/approval frontend that sits in front of Sonarr,
//! Radarr, and Plex. It exposes a REST API documented at
//! `api.overseerr.dev` (OpenAPI spec mirrored under `docs/upstream-api/`).
//!
//! Auth is header-token: `X-Api-Key: <key>`.

/// Public request/response types (serde).
pub mod types;

/// `OverseerrError` (thiserror).
pub mod error;

/// `OverseerrClient` struct + operation impls.
pub mod client;

pub use client::OverseerrClient;
pub use error::OverseerrError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the overseerr module.
pub const META: PluginMeta = PluginMeta {
    name: "overseerr",
    display_name: "Overseerr",
    description: "Request and approval frontend for Plex, Sonarr, and Radarr",
    category: Category::Media,
    docs_url: "https://api.overseerr.dev/",
    required_env: &[
        EnvVar {
            name: "OVERSEERR_URL",
            description: "Base URL of the Overseerr instance",
            example: "http://localhost:5055",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "OVERSEERR_API_KEY",
            description: "API key from Overseerr Settings → General",
            example: "MTY4NzA...",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(5055),
    supports_multi_instance: false,
};

impl ServiceClient for OverseerrClient {
    fn name(&self) -> &'static str {
        "overseerr"
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
            Err(OverseerrError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(OverseerrError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(OverseerrError::Api(e)) => Err(e),
        }
    }
}
