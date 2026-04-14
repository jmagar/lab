//! qBittorrent torrent client.
//!
//! qBittorrent uses a WebUI API v2 with session-cookie auth.
//! The `SID` cookie is obtained by `POST /api/v2/auth/login` and must be
//! stored and passed on every subsequent request.

/// `QbittorrentClient` — torrent management methods.
pub mod client;

/// qBittorrent request/response types (serde).
pub mod types;

/// `QbittorrentError` (thiserror).
pub mod error;

pub use client::QbittorrentClient;
pub use error::QbittorrentError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the qbittorrent module.
pub const META: PluginMeta = PluginMeta {
    name: "qbittorrent",
    display_name: "qBittorrent",
    description: "BitTorrent download client",
    category: Category::Download,
    docs_url: "https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)",
    required_env: &[
        EnvVar {
            name: "QBITTORRENT_URL",
            description: "Base URL of the qBittorrent WebUI instance",
            example: "http://localhost:8080",
            secret: false,
        },
        EnvVar {
            name: "QBITTORRENT_PASSWORD",
            description: "WebUI password (username defaults to 'admin')",
            example: "adminadmin",
            secret: true,
        },
    ],
    optional_env: &[
        EnvVar {
            name: "QBITTORRENT_USERNAME",
            description: "WebUI username (defaults to 'admin')",
            example: "admin",
            secret: false,
        },
        EnvVar {
            name: "QBITTORRENT_SID",
            description: "Pre-obtained session cookie (SID=<value>) — skips login handshake",
            example: "SID=abc123def456",
            secret: true,
        },
    ],
    default_port: Some(8080),
};

use std::time::Instant;

use crate::core::{ApiError, ServiceClient, ServiceStatus};

impl ServiceClient for QbittorrentClient {
    fn name(&self) -> &'static str {
        "qbittorrent"
    }

    fn service_type(&self) -> &'static str {
        "download"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.transfer_info().await {
            Ok(_info) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: None,
            }),
            Err(QbittorrentError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                message: Some("auth failed — SID cookie may be expired".into()),
            }),
            Err(QbittorrentError::Api(ApiError::Network(e))) => Ok(ServiceStatus::unreachable(e)),
            Err(QbittorrentError::Api(e)) => Ok(ServiceStatus::degraded(e.to_string())),
            Err(QbittorrentError::CommandFailed(msg)) => Ok(ServiceStatus::degraded(msg)),
        }
    }
}
