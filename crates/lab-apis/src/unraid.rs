//! Unraid server GraphQL API client.
//!
//! Provides async methods for querying system information, metrics, array
//! status, Docker containers, and physical disks via the Unraid Connect
//! GraphQL endpoint (`{base_url}/graphql`).
//!
//! # Auth
//! `Auth::ApiKey { header: "X-API-Key", key }` — matches the Unraid API spec.
//!
//! # Rate limits
//! Unraid enforces approximately 100 requests per 10 seconds. No in-process
//! rate limiter is provided; callers must stay within this bound.

pub mod client;
pub mod error;
pub mod types;

pub use client::UnraidClient;
pub use error::UnraidError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the `unraid` module.
pub const META: PluginMeta = PluginMeta {
    name: "unraid",
    display_name: "Unraid",
    description: "Unraid server GraphQL API — system info, metrics, array status, Docker, and disk management",
    category: Category::Network,
    docs_url: "https://docs.unraid.net/",
    required_env: &[
        EnvVar {
            name: "UNRAID_URL",
            description: "Base URL of the Unraid Connect API (e.g. https://10.0.0.2:31337)",
            example: "https://10.0.0.2:31337",
            secret: false,
        },
        EnvVar {
            name: "UNRAID_API_KEY",
            description: "Unraid API key (X-API-Key header)",
            example: "your-api-key",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(31337),
};

// ---------------------------------------------------------------------------
// ServiceClient impl
// ---------------------------------------------------------------------------

use crate::core::{error::ApiError, status::ServiceStatus, traits::ServiceClient};

impl ServiceClient for UnraidClient {
    fn name(&self) -> &'static str {
        "unraid"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    #[allow(clippy::cast_possible_truncation)]
    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = std::time::Instant::now();
        match self.system_online().await {
            Ok(online) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: start.elapsed().as_millis() as u64,
                message: if online {
                    Some("online".into())
                } else {
                    Some("server reports offline".into())
                },
            }),
            Err(UnraidError::Http(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some("authentication failed".into()),
            }),
            Err(e) => Ok(ServiceStatus::unreachable(e.to_string())),
        }
    }
}
