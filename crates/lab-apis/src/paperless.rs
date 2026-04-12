//! Paperless-ngx document management client.

/// `PaperlessClient` — document management methods.
pub mod client;

/// Paperless request/response types (serde).
pub mod types;

/// `PaperlessError` (thiserror).
pub mod error;

pub use client::PaperlessClient;
pub use error::PaperlessError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the paperless module.
pub const META: PluginMeta = PluginMeta {
    name: "paperless",
    display_name: "Paperless-ngx",
    description: "Self-hosted document management system with OCR and full-text search",
    category: Category::Documents,
    docs_url: "https://docs.paperless-ngx.com/api/",
    required_env: &[
        EnvVar {
            name: "PAPERLESS_URL",
            description: "Base URL of the Paperless-ngx instance",
            example: "http://localhost:8000",
            secret: false,
        },
        EnvVar {
            name: "PAPERLESS_TOKEN",
            description: "API token from Account → API Token",
            example: "abc123def456...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(8000),
};

impl ServiceClient for PaperlessClient {
    fn name(&self) -> &'static str {
        "paperless"
    }

    fn service_type(&self) -> &'static str {
        "documents"
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
            Err(PaperlessError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(PaperlessError::Api(ApiError::Network(msg))) => {
                Ok(ServiceStatus::unreachable(msg))
            }
            Err(PaperlessError::Api(e)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some(e.to_string()),
            }),
        }
    }
}
