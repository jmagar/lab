//! Uptime Kuma client contract.

use serde_json::json;

use crate::core::error::ApiError;
use crate::core::{Auth, HttpClient};

use super::error::UptimeKumaError;
use super::types::UptimeKumaResponse;

/// Client for a Uptime Kuma instance.
#[derive(Clone)]
pub struct UptimeKumaClient {
    http: HttpClient,
    has_credentials: bool,
}

impl UptimeKumaClient {
    /// Build a client against `base_url`.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UptimeKumaError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            has_credentials: false,
        })
    }

    /// Build a client with web-account credentials for the Socket.IO API.
    pub fn with_login(
        base_url: &str,
        username: String,
        password: String,
    ) -> Result<Self, UptimeKumaError> {
        if username.trim().is_empty() || password.is_empty() {
            return Err(ApiError::Internal(
                "UPTIME_KUMA_USERNAME and UPTIME_KUMA_PASSWORD must not be empty".into(),
            )
            .into());
        }
        Ok(Self {
            http: HttpClient::new(base_url, Auth::None)?,
            has_credentials: true,
        })
    }

    /// Probe the web UI root. Uptime Kuma does not expose a stable REST health endpoint.
    pub async fn health(&self) -> Result<(), UptimeKumaError> {
        self.http.get_text("/").await?;
        Ok(())
    }

    /// Return the supported integration contract.
    pub fn contract_status(&self) -> UptimeKumaResponse {
        UptimeKumaResponse {
            value: json!({
                "service": "uptime-kuma",
                "transport": "socket.io",
                "base_url": self.http.base_url(),
                "credentials_configured": self.has_credentials,
                "live_socket_reads": false,
                "read_actions": [
                    "monitor.list",
                    "monitor.get",
                    "heartbeat.list",
                    "status.summary",
                    "notification.list"
                ],
                "note": "Uptime Kuma's monitor data is exposed through an internal Socket.IO API; live socket reads are intentionally blocked until the actor is implemented."
            }),
        }
    }

    /// Placeholder for the future Socket.IO monitor inventory call.
    pub async fn unsupported_socket_read(
        &self,
        action: &str,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        Err(UptimeKumaError::Unsupported(format!(
            "{action} requires the Uptime Kuma Socket.IO actor"
        )))
    }
}
