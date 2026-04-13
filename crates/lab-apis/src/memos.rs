//! Memos client — self-hosted memo hub.
//!
//! Memos provides a clean v1 REST API for creating, listing, and managing
//! short-form notes (memos). Auth uses `Authorization: Bearer <access_token>`.

/// `MemosClient` — memo management methods.
pub mod client;

/// Memos request/response types (serde).
pub mod types;

/// `MemosError` (thiserror).
pub mod error;

pub use client::MemosClient;
pub use error::MemosError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the memos module.
pub const META: PluginMeta = PluginMeta {
    name: "memos",
    display_name: "Memos",
    description: "Lightweight self-hosted memo hub for short-form notes and journals",
    category: Category::Notes,
    docs_url: "https://www.usememos.com/docs/api-documentation",
    required_env: &[
        EnvVar {
            name: "MEMOS_URL",
            description: "Base URL of the Memos instance",
            example: "http://localhost:5230",
            secret: false,
        },
        EnvVar {
            name: "MEMOS_TOKEN",
            description: "Access token from Memos Settings → My Account → Access Tokens",
            example: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(5230),
};

impl ServiceClient for MemosClient {
    fn name(&self) -> &'static str {
        "memos"
    }

    fn service_type(&self) -> &'static str {
        "notes"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.workspace_profile().await {
            Ok(_) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(MemosError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed".into()),
            }),
            Err(MemosError::Api(ApiError::Network(msg))) => Ok(ServiceStatus::unreachable(msg)),
            Err(MemosError::Api(e)) => Err(e),
        }
    }
}
