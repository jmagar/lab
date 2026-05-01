//! FreshRSS Google Reader-compatible API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::FreshrssClient;
pub use error::FreshrssError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, TEXT_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "freshrss",
    display_name: "Freshrss",
    description: "FreshRSS feed reader via Google Reader API",
    category: Category::Notes,
    docs_url: "https://freshrss.github.io/FreshRSS/en/developers/06_GoogleReader_API.html",
    required_env: &[
        EnvVar {
            name: "FRESHRSS_URL",
            description: "FreshRSS Google Reader API base URL",
            example: "https://rss.example.com/api/greader.php",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "FRESHRSS_USERNAME",
            description: "FreshRSS username",
            example: "admin",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "FRESHRSS_API_PASSWORD",
            description: "FreshRSS API password",
            example: "api-password",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    optional_env: &[],
    default_port: Some(80),
    supports_multi_instance: false,
};

impl ServiceClient for FreshrssClient {
    fn name(&self) -> &'static str {
        "freshrss"
    }

    fn service_type(&self) -> &'static str {
        "notes"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.subscriptions()
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
