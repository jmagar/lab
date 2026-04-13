//! Linkding client — self-hosted bookmark manager.
//!
//! Linkding provides a clean REST API for managing bookmarks with tag support.
//! Auth uses `Authorization: Token <api_token>`.

/// `LinkdingClient` — bookmark management methods.
pub mod client;

/// Linkding request/response types (serde).
pub mod types;

/// `LinkdingError` (thiserror).
pub mod error;

pub use client::LinkdingClient;
pub use error::LinkdingError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the linkding module.
pub const META: PluginMeta = PluginMeta {
    name: "linkding",
    display_name: "Linkding",
    description: "Self-hosted bookmark manager with tagging and search",
    category: Category::Notes,
    docs_url: "https://github.com/sissbruecker/linkding/blob/master/docs/API.md",
    required_env: &[
        EnvVar {
            name: "LINKDING_URL",
            description: "Base URL of the Linkding instance",
            example: "http://localhost:9090",
            secret: false,
        },
        EnvVar {
            name: "LINKDING_TOKEN",
            description: "REST API token from the Linkding Settings page",
            example: "abc123def456...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(9090),
};

impl ServiceClient for LinkdingClient {
    fn name(&self) -> &'static str {
        "linkding"
    }

    fn service_type(&self) -> &'static str {
        "notes"
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
            Err(LinkdingError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(LinkdingError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(LinkdingError::Api(e)) => Err(e),
        }
    }
}
