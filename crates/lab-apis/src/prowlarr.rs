//! Prowlarr — Indexer manager for the Servarr stack.

/// `ProwlarrClient` — indexer manager methods.
pub mod client;

/// Prowlarr request/response types (serde).
pub mod types;

/// `ProwlarrError` (thiserror).
pub mod error;

pub use client::ProwlarrClient;
pub use error::ProwlarrError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the prowlarr module.
pub const META: PluginMeta = PluginMeta {
    name: "prowlarr",
    display_name: "Prowlarr",
    description: "Indexer manager for the Servarr stack",
    category: Category::Indexer,
    docs_url: "https://prowlarr.com/docs/api/",
    required_env: &[
        EnvVar {
            name: "PROWLARR_URL",
            description: "Base URL for the Prowlarr instance",
            example: "http://prowlarr:9696",
            secret: false,
        },
        EnvVar {
            name: "PROWLARR_API_KEY",
            description: "Prowlarr API key (Settings → General → API Key)",
            example: "abc123def456",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(9696),
};

impl ServiceClient for ProwlarrClient {
    fn name(&self) -> &'static str {
        "prowlarr"
    }

    fn service_type(&self) -> &'static str {
        "indexer"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.health().await {
            Ok(()) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(ProwlarrError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(ProwlarrError::Api(ApiError::Network(e))) => {
                Ok(ServiceStatus::unreachable(e))
            }
            Err(ProwlarrError::Api(e)) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
