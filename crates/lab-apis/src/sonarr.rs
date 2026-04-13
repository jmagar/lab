//! Sonarr — TV series management for the Servarr stack.

/// `SonarrClient` — TV series management methods.
pub mod client;

/// Sonarr request/response types (serde).
pub mod types;

/// `SonarrError` (thiserror).
pub mod error;

pub use client::SonarrClient;
pub use error::SonarrError;

use std::time::{Duration, Instant};

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the sonarr module.
pub const META: PluginMeta = PluginMeta {
    name: "sonarr",
    display_name: "Sonarr",
    description: "TV series management for the Servarr stack",
    category: Category::Servarr,
    docs_url: "https://sonarr.tv/docs/api/",
    required_env: &[
        EnvVar {
            name: "SONARR_URL",
            description: "Base URL for the Sonarr instance",
            example: "http://sonarr:8989",
            secret: false,
        },
        EnvVar {
            name: "SONARR_API_KEY",
            description: "Sonarr API key (Settings → General → API Key)",
            example: "abc123def456",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(8989),
};

impl ServiceClient for SonarrClient {
    fn name(&self) -> &'static str {
        "sonarr"
    }

    fn service_type(&self) -> &'static str {
        "servarr"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        let probe = tokio::time::timeout(Duration::from_secs(5), self.probe()).await;
        match probe {
            Err(_elapsed) => Ok(ServiceStatus {
                reachable: false,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("health check timed out".into()),
            }),
            Ok(Ok(())) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Ok(Err(SonarrError::Api(ApiError::Auth))) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Ok(Err(SonarrError::Api(ApiError::Network(e)))) => Ok(ServiceStatus::unreachable(e)),
            Ok(Err(SonarrError::Api(e))) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
