//! Pi-hole service module.

pub mod client;
pub mod error;
pub mod types;

pub use client::PiholeClient;
pub use error::PiholeError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, SECRET_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "pihole",
    display_name: "Pi-hole",
    description: "Pi-hole v6 DNS sinkhole API",
    category: Category::Network,
    docs_url: "https://docs.pi-hole.net/api/",
    required_env: &[
        EnvVar {
            name: "PIHOLE_URL",
            description: "Base URL for Pi-hole",
            example: "http://localhost",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "PIHOLE_PASSWORD",
            description: "Pi-hole web/API password",
            example: "change-me",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[EnvVar {
        name: "PIHOLE_TOTP",
        description: "Optional Pi-hole TOTP code for two-factor auth",
        example: "123456",
        secret: true,
        ui: Some(&SECRET_OPTIONAL_FIELD),
    }],
    default_port: Some(80),
    supports_multi_instance: false,
};

impl ServiceClient for PiholeClient {
    fn name(&self) -> &'static str {
        "pihole"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.blocking_status()
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
