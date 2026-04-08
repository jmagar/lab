//! UniFi Network Application local API client.
//!
//! UniFi exposes a local integration API under `/proxy/network/integration/v1`.
//! This module follows the same pattern as the Radarr SDK: a small public
//! module that re-exports the client and error type, plus a compile-time
//! `META` block used by the binary.

/// Public request/response types (serde).
pub mod types;

/// `UnifiError` (thiserror).
pub mod error;

/// `UnifiClient` — read-only network inspection methods.
pub mod client;

pub use client::UnifiClient;
pub use error::UnifiError;

use std::time::Instant;

use crate::core::{
    ApiError, ServiceClient,
    plugin::{Category, EnvVar, PluginMeta},
};

/// Compile-time metadata for the unifi module.
pub const META: PluginMeta = PluginMeta {
    name: "unifi",
    display_name: "UniFi",
    description: "UniFi Network Application local API",
    category: Category::Network,
    docs_url: "https://ubntwiki.com/products/software/unifi-controller/api",
    required_env: &[
        EnvVar {
            name: "UNIFI_URL",
            description: "Base URL of the UniFi controller",
            example: "https://10.1.0.1",
            secret: false,
        },
        EnvVar {
            name: "UNIFI_API_KEY",
            description: "API key from UniFi Integrations",
            example: "abc123def456...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(443),
};

impl ServiceClient for UnifiClient {
    fn name(&self) -> &'static str {
        "unifi"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<crate::core::ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.info().await {
            Ok(info) => Ok(crate::core::ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: Some(info.application_version),
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(UnifiError::Api(ApiError::Auth)) => Ok(crate::core::ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(UnifiError::Api(e)) => Ok(crate::core::ServiceStatus::unreachable(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::META;

    #[test]
    fn metadata_requires_url_and_api_key() {
        let required: Vec<_> = META.required_env.iter().map(|v| v.name).collect();
        assert!(required.contains(&"UNIFI_URL"));
        assert!(required.contains(&"UNIFI_API_KEY"));
    }
}
