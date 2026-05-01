//! Beads issue tracker local CLI client.

pub mod client;
pub mod error;
pub mod types;

pub use client::BeadsClient;
pub use error::BeadsError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::TEXT_FIELD;
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "beads",
    display_name: "Beads",
    description: "Git/Dolt-backed issue tracker through the local bd CLI",
    category: Category::Bootstrap,
    docs_url: "https://gastownhall.github.io/beads/",
    required_env: &[],
    optional_env: &[EnvVar {
        name: "BEADS_BIN",
        description: "Path or command name for the bd executable",
        example: "bd",
        secret: false,
        ui: Some(&TEXT_FIELD),
    }],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for BeadsClient {
    fn name(&self) -> &'static str {
        "beads"
    }

    fn service_type(&self) -> &'static str {
        "bootstrap"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let status = self
            .health_status()
            .await
            .map_err(|err| ApiError::Internal(err.to_string()))?;
        Ok(ServiceStatus {
            reachable: status.bd_available,
            auth_ok: true,
            version: status.version,
            latency_ms: 0,
            message: status.message,
        })
    }
}
