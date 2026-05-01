//! AdGuard Home read-first API client.

pub mod client;
pub mod error;
pub mod types;

pub use client::AdguardClient;
pub use error::AdguardError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, SECRET_OPTIONAL_FIELD, TEXT_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "adguard",
    display_name: "Adguard",
    description: "AdGuard Home DNS filtering and query-log summaries",
    category: Category::Network,
    docs_url: "https://github.com/AdguardTeam/AdGuardHome/blob/master/AGHTechDoc.md",
    required_env: &[EnvVar {
        name: "ADGUARD_URL",
        description: "Base URL for AdGuard Home",
        example: "http://localhost:3000",
        secret: false,
        ui: Some(&URL_FIELD),
    }],
    optional_env: &[
        EnvVar {
            name: "ADGUARD_SESSION_COOKIE",
            description: "Optional pre-authenticated AdGuard Home session cookie",
            example: "agh_session=...",
            secret: true,
            ui: Some(&SECRET_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "ADGUARD_USERNAME",
            description: "AdGuard Home username for cookie-session login",
            example: "admin",
            secret: false,
            ui: Some(&TEXT_FIELD),
        },
        EnvVar {
            name: "ADGUARD_PASSWORD",
            description: "AdGuard Home password for cookie-session login",
            example: "password",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
    ],
    default_port: Some(3000),
    supports_multi_instance: false,
};

impl ServiceClient for AdguardClient {
    fn name(&self) -> &'static str {
        "adguard"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.status()
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
