//! Immich read-only API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::ImmichClient;
pub use error::ImmichError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "immich",
    display_name: "Immich",
    description: "Self-hosted photo and video metadata",
    category: Category::Media,
    docs_url: "https://api.immich.app/",
    required_env: &[
        EnvVar {
            name: "IMMICH_URL",
            description: "Base URL for the Immich server",
            example: "http://localhost:2283",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "IMMICH_API_KEY",
            description: "Immich API key",
            example: "immich_api_key",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(2283),
    supports_multi_instance: false,
};

impl ServiceClient for ImmichClient {
    fn name(&self) -> &'static str {
        "immich"
    }

    fn service_type(&self) -> &'static str {
        "media"
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
