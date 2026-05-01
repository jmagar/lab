//! Scaffolded lab-apis service module for uptime_kuma.

pub mod client;
pub mod error;
pub mod types;

pub use client::UptimeKumaClient;
pub use error::UptimeKumaError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the scaffolded service.
pub const META: PluginMeta = PluginMeta {
    name: "uptime_kuma",
    display_name: "UptimeKuma",
    description: "Scaffolded service placeholder",
    category: Category::Bootstrap,
    docs_url: "https://example.invalid",
    required_env: &[EnvVar {
        name: "UPTIME_KUMA_URL",
        description: "Base URL for the UptimeKuma service",
        example: "http://localhost:8080",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[EnvVar {
        name: "UPTIME_KUMA_API_KEY",
        description: "Optional API key for the UptimeKuma service",
        example: "abc123...",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for UptimeKumaClient {
    fn name(&self) -> &'static str {
        "uptime_kuma"
    }

    fn service_type(&self) -> &'static str {
        "bootstrap"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        Ok(ServiceStatus::degraded("scaffolded service placeholder"))
    }
}
