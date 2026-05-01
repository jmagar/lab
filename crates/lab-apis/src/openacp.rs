//! OpenACP daemon client.
//!
//! This module targets the upstream OpenACP REST daemon. It is distinct from
//! Lab's internal `acp` capability service.

/// `OpenAcpClient` HTTP client.
pub mod client;
/// OpenACP-specific errors.
pub mod error;
/// Request and response types for the OpenACP REST API.
pub mod types;

pub use client::OpenAcpClient;
pub use error::OpenAcpError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the OpenACP module.
pub const META: PluginMeta = PluginMeta {
    name: "openacp",
    display_name: "OpenACP",
    description: "Upstream OpenACP daemon for agent sessions and messaging adapters",
    category: Category::Ai,
    docs_url: "https://openacp.ai/",
    required_env: &[
        EnvVar {
            name: "OPENACP_URL",
            description: "Base URL of the OpenACP daemon",
            example: "http://127.0.0.1:21420",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "OPENACP_TOKEN",
            description: "OpenACP bearer token or scoped JWT",
            example: "0123456789abcdef...",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(21420),
    supports_multi_instance: true,
};

impl ServiceClient for OpenAcpClient {
    fn name(&self) -> &'static str {
        "openacp"
    }

    fn service_type(&self) -> &'static str {
        "ai"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.health().await {
            Ok(health) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: health
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: health
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
            }),
            Err(OpenAcpError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(OpenAcpError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(OpenAcpError::Api(e)) => Err(e),
        }
    }
}
