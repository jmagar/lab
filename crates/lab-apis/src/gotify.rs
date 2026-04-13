//! Gotify push-notification server client.
//!
//! Gotify exposes a small REST API (Swagger 2.0) for sending messages,
//! managing applications, and managing clients. Auth is a header token —
//! `X-Gotify-Key` — scoped per-app (send) or per-client (read/manage).
//!
//! Spec: `docs/upstream-api/gotify.openapi.json`.

/// Public request/response types (serde).
pub mod types;

/// `GotifyError` (thiserror).
pub mod error;

/// `GotifyClient` — send messages, manage apps/clients/messages.
pub mod client;

pub use client::GotifyClient;
pub use error::GotifyError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the gotify module.
pub const META: PluginMeta = PluginMeta {
    name: "gotify",
    display_name: "Gotify",
    description: "Self-hosted push notification server",
    category: Category::Notifications,
    docs_url: "https://gotify.net/api-docs",
    required_env: &[
        EnvVar {
            name: "GOTIFY_URL",
            description: "Base URL of the Gotify server",
            example: "http://localhost:8080",
            secret: false,
        },
        EnvVar {
            name: "GOTIFY_TOKEN",
            description: "Token for Gotify API access; used as fallback when scoped tokens are absent",
            example: "A1b2C3d4E5...",
            secret: true,
        },
    ],
    optional_env: &[
        EnvVar {
            name: "GOTIFY_APP_TOKEN",
            description: "App token used by message.send (overrides GOTIFY_TOKEN for sending)",
            example: "A1b2C3d4E5...",
            secret: true,
        },
        EnvVar {
            name: "GOTIFY_CLIENT_TOKEN",
            description: "Client token used by message/app/client management actions (overrides GOTIFY_TOKEN)",
            example: "A1b2C3d4E5...",
            secret: true,
        },
    ],
    default_port: Some(80),
};

impl ServiceClient for GotifyClient {
    fn name(&self) -> &'static str {
        "gotify"
    }

    fn service_type(&self) -> &'static str {
        "notifications"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = std::time::Instant::now();
        match self.server_health().await {
            Ok(h) => {
                let healthy = h.health == "green" && h.database == "green";
                Ok(ServiceStatus {
                    reachable: true,
                    auth_ok: true,
                    version: None,
                    latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                    message: if healthy {
                        None
                    } else {
                        Some(format!("health={}, database={}", h.health, h.database))
                    },
                })
            }
            Err(GotifyError::Api(api)) => match api.kind() {
                "auth_failed" => Ok(ServiceStatus {
                    reachable: true,
                    auth_ok: false,
                    version: None,
                    latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                    message: Some("authentication failed".to_string()),
                }),
                "network_error" => Ok(ServiceStatus::unreachable(api.to_string())),
                _ => Err(api),
            },
        }
    }
}
