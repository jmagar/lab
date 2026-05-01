//! Uptime Kuma service module.

pub mod client;
pub mod error;
pub mod types;

pub use client::UptimeKumaClient;
pub use error::UptimeKumaError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, TEXT_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "uptime-kuma",
    display_name: "Uptime Kuma",
    description: "Self-hosted uptime monitor with a Socket.IO-backed API",
    category: Category::Network,
    docs_url: "https://github.com/louislam/uptime-kuma/wiki/API-Documentation",
    required_env: &[
        EnvVar {
            name: "UPTIME_KUMA_URL",
            description: "Base URL for Uptime Kuma",
            example: "http://localhost:3001",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "UPTIME_KUMA_USERNAME",
            description: "Username for the Uptime Kuma web account",
            example: "admin",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "UPTIME_KUMA_PASSWORD",
            description: "Password for the Uptime Kuma web account",
            example: "change-me",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(3001),
    supports_multi_instance: false,
};

impl ServiceClient for UptimeKumaClient {
    fn name(&self) -> &'static str {
        "uptime-kuma"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.health()
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(ServiceStatus {
            reachable: true,
            auth_ok: true,
            version: None,
            latency_ms: 0,
            message: None,
        })
    }
}
