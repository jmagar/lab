//! LoggiFly source-contract placeholder.

pub mod client;
pub mod error;
pub mod types;

pub use client::LoggiflyClient;
pub use error::LoggiflyError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{TEXT_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "loggifly",
    display_name: "LoggiFly",
    description: "Docker log/event monitor source-contract status",
    category: Category::Network,
    docs_url: "https://clemcer.github.io/LoggiFly/",
    required_env: &[],
    optional_env: &[
        EnvVar {
            name: "LOGGIFLY_DOCS_URL",
            description: "Source documentation URL used for contract review",
            example: "https://clemcer.github.io/LoggiFly/",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "LOGGIFLY_CONFIG_ROOT",
            description: "Future allowlisted root for config/health inspection",
            example: "/etc/loggifly",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
    ],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for LoggiflyClient {
    fn name(&self) -> &'static str {
        "loggifly"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        Ok(ServiceStatus::degraded(
            "implementation deferred: no stable safe API",
        ))
    }
}
