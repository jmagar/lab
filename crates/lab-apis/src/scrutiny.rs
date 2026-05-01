//! Scrutiny read-only API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::ScrutinyClient;
pub use error::ScrutinyError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "scrutiny",
    display_name: "Scrutiny",
    description: "SMART disk health summaries",
    category: Category::Network,
    docs_url: "https://github.com/AnalogJ/scrutiny",
    required_env: &[EnvVar {
        name: "SCRUTINY_URL",
        description: "Base URL for the Scrutiny web UI",
        example: "http://localhost:8080",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[
        EnvVar {
            name: "SCRUTINY_API_KEY",
            description: "Optional bearer token or reverse-proxy token",
            example: "abc123...",
            secret: true,
            ui: Some(&SECRET_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "SCRUTINY_TOKEN",
            description: "Alias for optional bearer token or reverse-proxy token",
            example: "abc123...",
            secret: true,
            ui: Some(&SECRET_OPTIONAL_FIELD),
        },
    ],
    default_port: Some(8080),
    supports_multi_instance: false,
};

impl ServiceClient for ScrutinyClient {
    fn name(&self) -> &'static str {
        "scrutiny"
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
