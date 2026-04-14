//! Tailscale WireGuard-based mesh VPN client.
//!
//! Auth: Bearer token (`tskey-api-*`).
//! Spec: `docs/upstream-api/tailscale.openapi.yaml`.

/// Public request/response types (serde).
pub mod types;

/// `TailscaleError` (thiserror).
pub mod error;

/// `TailscaleClient` — list devices, manage auth keys, query DNS.
pub mod client;

pub use client::TailscaleClient;
pub use error::TailscaleError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the tailscale module.
pub const META: PluginMeta = PluginMeta {
    name: "tailscale",
    display_name: "Tailscale",
    description: "WireGuard-based mesh VPN — list devices, manage auth keys, query DNS settings",
    category: Category::Network,
    docs_url: "https://tailscale.com/api",
    required_env: &[EnvVar {
        name: "TAILSCALE_API_KEY",
        description: "Tailscale API access token (tskey-api-*)",
        example: "tskey-api-xxxxx",
        secret: true,
    }],
    optional_env: &[
        EnvVar {
            name: "TAILSCALE_TAILNET",
            description: "Tailnet name (e.g. example.com); defaults to \"-\" (auth key's default tailnet)",
            example: "example.com",
            secret: false,
        },
        EnvVar {
            name: "TAILSCALE_BASE_URL",
            description: "Override the Tailscale API base URL (useful for self-hosted or testing)",
            example: "https://api.tailscale.com/api/v2",
            secret: false,
        },
    ],
    default_port: None,
};

impl ServiceClient for TailscaleClient {
    fn name(&self) -> &'static str {
        "tailscale"
    }

    fn service_type(&self) -> &'static str {
        "network"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = std::time::Instant::now();
        match self.devices_list().await {
            Ok(_) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(TailscaleError::Api(api)) => match api.kind() {
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
