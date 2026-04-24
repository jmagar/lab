//! ACP Agent Registry client — discover ACP-compatible AI coding agents.
//!
//! This module wraps the read-only CDN endpoint at
//! <https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json>.
//! No auth required; all reads are unauthenticated.
//! The only env var is `ACP_REGISTRY_URL` (optional; defaults to the official CDN).

pub mod client;
pub mod error;
pub mod types;

pub use client::AcpRegistryClient;
pub use error::AcpRegistryError;

use std::time::Instant;

use crate::core::plugin::{Category, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the acp_registry module.
pub const META: PluginMeta = PluginMeta {
    name: "acp_registry",
    display_name: "ACP Registry",
    description: "ACP Agent Registry — discover and install ACP-compatible AI coding agents",
    category: Category::Marketplace,
    docs_url: "https://agentclientprotocol.com",
    required_env: &[],
    optional_env: &[],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for AcpRegistryClient {
    fn name(&self) -> &'static str {
        "acp_registry"
    }

    fn service_type(&self) -> &'static str {
        "marketplace"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.health_probe().await {
            Ok(_) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(AcpRegistryError::Request(e)) => Ok(ServiceStatus::unreachable(e.to_string())),
            Err(e) => Ok(ServiceStatus::unreachable(e.to_string())),
        }
    }
}
