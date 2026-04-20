//! MCP Registry client — browse and search the official MCP server registry.
//!
//! This module wraps the read-only v0.1 endpoints at
//! <https://registry.modelcontextprotocol.io>. No auth required; all reads are
//! unauthenticated. The only env var is `MCPREGISTRY_URL` (optional; defaults to
//! the official registry).

pub mod client;
pub mod error;
pub mod types;

pub use client::McpRegistryClient;
pub use error::RegistryError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the mcpregistry module.
pub const META: PluginMeta = PluginMeta {
    name: "mcpregistry",
    display_name: "MCP Registry",
    description: "Browse and search the official MCP server registry",
    category: Category::Bootstrap,
    docs_url: "https://registry.modelcontextprotocol.io",
    required_env: &[],
    optional_env: &[EnvVar {
        name: "MCPREGISTRY_URL",
        description: "MCP Registry base URL",
        example: "https://registry.modelcontextprotocol.io",
        secret: false,
    }],
    default_port: None,
};

impl ServiceClient for McpRegistryClient {
    fn name(&self) -> &'static str {
        "mcpregistry"
    }

    fn service_type(&self) -> &'static str {
        "bootstrap"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.health_probe().await {
            Ok(body) if body.status == "ok" => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Ok(body) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some(format!("registry status: {}", body.status)),
            }),
            Err(RegistryError::Api(e)) => Ok(ServiceStatus::unreachable(e.to_string())),
            Err(e) => Ok(ServiceStatus::unreachable(e.to_string())),
        }
    }
}
