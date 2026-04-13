//! Arcane Docker management UI client.

/// `ArcaneClient` — Docker management methods.
pub mod client;

/// Arcane request/response types (serde).
pub mod types;

/// `ArcaneError` (thiserror).
pub mod error;

pub use client::ArcaneClient;
pub use error::ArcaneError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the arcane module.
pub const META: PluginMeta = PluginMeta {
    name: "arcane",
    display_name: "Arcane",
    description: "Docker management UI — environments, containers, images, and volumes",
    category: Category::Network,
    docs_url: "https://github.com/arcane-app/arcane",
    required_env: &[
        EnvVar {
            name: "ARCANE_URL",
            description: "Base URL for the Arcane API (e.g. http://arcane:3000)",
            example: "http://arcane:3000",
            secret: false,
        },
        EnvVar {
            name: "ARCANE_API_KEY",
            description: "API key for authentication (X-API-Key header)",
            example: "arck_xxx",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(3000),
};

impl ServiceClient for ArcaneClient {
    fn name(&self) -> &'static str {
        "arcane"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = std::time::Instant::now();
        match self.health().await {
            Ok(resp) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some(resp.status),
            }),
            Err(ArcaneError::Api(e)) => match &e {
                ApiError::Network(_) => Ok(ServiceStatus::unreachable(e.to_string())),
                _ => Ok(ServiceStatus::degraded(e.to_string())),
            },
        }
    }
}
