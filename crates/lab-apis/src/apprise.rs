//! Apprise notification dispatcher client.
//!
//! Apprise is a universal notification library that speaks to 100+ upstream
//! services (Slack, Discord, Telegram, Gotify, Pushover, email, webhooks,
//! etc.) behind a single unified URL scheme (e.g. `slack://`, `tgram://`).
//!
//! This module targets `apprise-api` — the Flask HTTP wrapper maintained at
//! `github.com/caronc/apprise-api`. It exposes a small REST surface:
//!
//! - `POST /notify` — stateless: caller supplies URLs + payload in one call
//! - `POST /notify/{key}` — stateful: send via URLs persisted under `key`
//! - `POST /add/{key}` — persist a config (URL list or YAML/TEXT blob)
//! - `GET  /get/{key}` — retrieve the config for a key
//! - `POST /del/{key}` — delete a key
//!
//! Spec: `docs/upstream-api/apprise.md` (mirrored from the upstream README —
//! apprise-api does not publish an OpenAPI document).
//!
//! Auth is optional: apprise-api can run unauthenticated, or behind a reverse
//! proxy that injects basic-auth / bearer headers.

/// Public request/response types (serde).
pub mod types;

/// `AppriseError` (thiserror).
pub mod error;

/// `AppriseClient` — notify, persist configs, dispatch.
pub mod client;

pub use client::AppriseClient;
pub use error::AppriseError;

use std::time::Instant;

use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::{ApiError, ServiceClient, ServiceStatus};

/// Compile-time metadata for the apprise module.
pub const META: PluginMeta = PluginMeta {
    name: "apprise",
    display_name: "Apprise",
    description: "Universal notification dispatcher (100+ services behind one URL scheme)",
    category: Category::Notifications,
    docs_url: "https://github.com/caronc/apprise-api",
    required_env: &[EnvVar {
        name: "APPRISE_URL",
        description: "Base URL of the apprise-api server",
        example: "http://localhost:8000",
        secret: false,
    }],
    optional_env: &[EnvVar {
        name: "APPRISE_TOKEN",
        description: "Bearer token if behind auth proxy",
        example: "abc123...",
        secret: true,
    }],
    default_port: Some(8000),
};

impl ServiceClient for AppriseClient {
    fn name(&self) -> &'static str {
        "apprise"
    }

    fn service_type(&self) -> &'static str {
        "notifications"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match AppriseClient::health(self).await {
            Ok(()) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(AppriseError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("authentication failed".into()),
            }),
            Err(AppriseError::Api(ApiError::Network(err))) => Ok(ServiceStatus::unreachable(err)),
            Err(AppriseError::Api(err)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some(err.to_string()),
            }),
        }
    }
}
