//! Scaffolded lab-apis service module for {{service}}.

pub mod client;
pub mod error;
pub mod types;

pub use client::{{Service}}Client;
pub use error::{{Service}}Error;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the scaffolded service.
pub const META: PluginMeta = PluginMeta {
    name: "{{service}}",
    display_name: "{{Service}}",
    description: "Scaffolded service placeholder",
    category: Category::Bootstrap,
    docs_url: "https://example.invalid",
    required_env: &[EnvVar {
        name: "{{SERVICE}}_URL",
        description: "Base URL for the {{Service}} service",
        example: "http://localhost:8080",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[EnvVar {
        name: "{{SERVICE}}_API_KEY",
        description: "Optional API key for the {{Service}} service",
        example: "abc123...",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for {{Service}}Client {
    fn name(&self) -> &'static str {
        "{{service}}"
    }

    fn service_type(&self) -> &'static str {
        "bootstrap"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        Ok(ServiceStatus::degraded("scaffolded service placeholder"))
    }
}
