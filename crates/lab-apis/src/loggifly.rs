//! LoggiFly local heartbeat/config contract.

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
    description: "Docker log/event monitor local heartbeat/config status",
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
            description: "Allowlisted directory containing LoggiFly config.yaml for redacted summary",
            example: "/etc/loggifly",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "LOGGIFLY_HEARTBEAT_PATH",
            description: "Heartbeat file written by LoggiFly when ENABLE_HEALTHCHECK=true",
            example: "/dev/shm/loggifly-heartbeat",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "LOGGIFLY_HEARTBEAT_INTERVAL_SECS",
            description: "Heartbeat interval in seconds; max age is interval * 1.5",
            example: "60",
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
        let status = self
            .health_status()
            .await
            .map_err(|err| ApiError::Internal(err.to_string()))?;
        match status.status {
            "healthy" => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: 0,
                message: Some("heartbeat fresh".into()),
            }),
            "stale" => Ok(ServiceStatus::degraded("heartbeat stale")),
            "missing" => Ok(ServiceStatus::degraded("heartbeat missing")),
            other => Ok(ServiceStatus::degraded(format!("heartbeat {other}"))),
        }
    }
}
