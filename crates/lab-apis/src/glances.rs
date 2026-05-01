//! Glances REST API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::GlancesClient;
pub use error::GlancesError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "glances",
    display_name: "Glances",
    description: "Host metrics via Glances REST API v4",
    category: Category::Network,
    docs_url: "https://glances.readthedocs.io/en/latest/api/restful.html",
    required_env: &[EnvVar {
        name: "GLANCES_URL",
        description: "Base URL for Glances",
        example: "http://localhost:61208",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[EnvVar {
        name: "GLANCES_TOKEN",
        description: "Optional JWT bearer token for Glances",
        example: "jwt...",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: Some(61208),
    supports_multi_instance: false,
};

impl ServiceClient for GlancesClient {
    fn name(&self) -> &'static str {
        "glances"
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
