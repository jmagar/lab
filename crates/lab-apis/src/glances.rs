//! Scaffolded lab-apis service module for glances.

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

/// Compile-time metadata for the scaffolded service.
pub const META: PluginMeta = PluginMeta {
    name: "glances",
    display_name: "Glances",
    description: "Scaffolded service placeholder",
    category: Category::Bootstrap,
    docs_url: "https://example.invalid",
    required_env: &[EnvVar {
        name: "GLANCES_URL",
        description: "Base URL for the Glances service",
        example: "http://localhost:8080",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[EnvVar {
        name: "GLANCES_API_KEY",
        description: "Optional API key for the Glances service",
        example: "abc123...",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for GlancesClient {
    fn name(&self) -> &'static str {
        "glances"
    }

    fn service_type(&self) -> &'static str {
        "bootstrap"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        Ok(ServiceStatus::degraded("scaffolded service placeholder"))
    }
}
