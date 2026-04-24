//! Tautulli Plex analytics client.
//!
//! Tautulli monitors Plex Media Server activity, providing history, statistics,
//! user management, and notification features via a REST API.
//!
//! Auth: `apikey` query parameter on every request.

/// `TautulliClient` — Tautulli methods.
pub mod client;

/// Tautulli request/response types (serde).
pub mod types;

/// `TautulliError` (thiserror).
pub mod error;

pub use client::TautulliClient;
pub use error::TautulliError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the tautulli module.
pub const META: PluginMeta = PluginMeta {
    name: "tautulli",
    display_name: "Tautulli",
    description: "Plex Media Server analytics — activity, history, statistics, and user management",
    category: Category::Media,
    docs_url: "https://github.com/Tautulli/Tautulli/wiki/Tautulli-API-Reference",
    required_env: &[
        EnvVar {
            name: "TAUTULLI_URL",
            description: "Base URL of the Tautulli instance",
            example: "http://localhost:8181",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "TAUTULLI_API_KEY",
            description: "Tautulli API key (Settings → Web Interface → API Key)",
            example: "abcdef1234567890abcdef1234567890",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(8181),
    supports_multi_instance: false,
};

impl ServiceClient for TautulliClient {
    fn name(&self) -> &'static str {
        "tautulli"
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
            Err(TautulliError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(TautulliError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(TautulliError::Api(e)) => Err(e),
        }
    }
}
