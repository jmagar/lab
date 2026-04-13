//! Scaffolded lab-apis service module for {{service}}.

pub mod client;
pub mod error;
pub mod types;

pub use client::{{Service}}Client;
pub use error::{{Service}}Error;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, PluginMeta};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the scaffolded service.
pub const META: PluginMeta = PluginMeta {
    name: "{{service}}",
    display_name: "{{Service}}",
    description: "Scaffolded service placeholder",
    category: Category::Bootstrap,
    docs_url: "https://example.invalid",
    required_env: &[],
    optional_env: &[],
    default_port: None,
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
