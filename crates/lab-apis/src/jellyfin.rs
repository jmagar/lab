//! Jellyfin media server client.
//!
//! First-wave support is read-heavy: system information, users, libraries,
//! items, sessions, and plugins.

pub mod client;
pub mod error;
pub mod types;

pub use client::JellyfinClient;
pub use error::JellyfinError;

use std::time::Instant;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the Jellyfin module.
pub const META: PluginMeta = PluginMeta {
    name: "jellyfin",
    display_name: "Jellyfin",
    description: "Jellyfin media server inventory and operator status",
    category: Category::Media,
    docs_url: "https://api.jellyfin.org/",
    required_env: &[
        EnvVar {
            name: "JELLYFIN_URL",
            description: "Base URL of the Jellyfin server",
            example: "http://localhost:8096",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "JELLYFIN_API_KEY",
            description: "Jellyfin API key for the Authorization header",
            example: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(8096),
    supports_multi_instance: true,
};

impl ServiceClient for JellyfinClient {
    fn name(&self) -> &'static str {
        "jellyfin"
    }

    fn service_type(&self) -> &'static str {
        "media"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.system_info().await {
            Ok(info) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: info.version,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: info.server_name,
            }),
            Err(JellyfinError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(JellyfinError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(JellyfinError::Api(e)) => Err(e),
            Err(e) => Err(ApiError::Internal(e.to_string())),
        }
    }
}
