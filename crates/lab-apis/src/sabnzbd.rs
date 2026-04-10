//! SABnzbd client — Usenet download management.
//!
//! SABnzbd uses a query-parameter API (`GET /api?mode=<action>&apikey=<key>&output=json`).
//! Auth is API key passed as a query param, not a header.

/// `SabnzbdClient` — Usenet download management methods.
pub mod client;

/// SABnzbd request/response types (serde).
pub mod types;

/// `SabnzbdError` (thiserror).
pub mod error;

pub use client::SabnzbdClient;
pub use error::SabnzbdError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the sabnzbd module.
pub const META: PluginMeta = PluginMeta {
    name: "sabnzbd",
    display_name: "SABnzbd",
    description: "Usenet download client",
    category: Category::Download,
    docs_url: "https://sabnzbd.org/wiki/configuration/4.0/api",
    required_env: &[
        EnvVar {
            name: "SABNZBD_URL",
            description: "Base URL of the SABnzbd instance",
            example: "http://localhost:8080",
            secret: false,
        },
        EnvVar {
            name: "SABNZBD_API_KEY",
            description: "API key from SABnzbd Config → General",
            example: "abc123def456...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(8080),
};

use std::time::Instant;

use crate::core::{ApiError, ServiceClient, ServiceStatus};

impl ServiceClient for SabnzbdClient {
    fn name(&self) -> &'static str {
        "sabnzbd"
    }

    fn service_type(&self) -> &'static str {
        "download"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.version().await {
            Ok(version) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: Some(version),
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(SabnzbdError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(SabnzbdError::Api(ApiError::Network(e))) => Ok(ServiceStatus::unreachable(e)),
            Err(SabnzbdError::Api(e)) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
