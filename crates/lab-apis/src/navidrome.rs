//! Navidrome Subsonic-compatible API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::NavidromeClient;
pub use error::NavidromeError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, TEXT_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "navidrome",
    display_name: "Navidrome",
    description: "Subsonic-compatible music library metadata",
    category: Category::Media,
    docs_url: "https://www.navidrome.org/docs/developers/subsonic-api/",
    required_env: &[
        EnvVar {
            name: "NAVIDROME_URL",
            description: "Base URL for the Navidrome server",
            example: "http://localhost:4533",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "NAVIDROME_USERNAME",
            description: "Subsonic username",
            example: "admin",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "NAVIDROME_TOKEN",
            description: "Precomputed Subsonic token",
            example: "md5_password_plus_salt",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
        EnvVar {
            name: "NAVIDROME_SALT",
            description: "Subsonic token salt",
            example: "randomsalt",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(4533),
    supports_multi_instance: false,
};

impl ServiceClient for NavidromeClient {
    fn name(&self) -> &'static str {
        "navidrome"
    }

    fn service_type(&self) -> &'static str {
        "media"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.ping()
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
