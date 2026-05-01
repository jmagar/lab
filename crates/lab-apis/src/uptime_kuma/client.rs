//! Scaffolded client for uptime_kuma.

use crate::core::{Auth, HttpClient};

use super::error::UptimeKumaError;

/// Scaffolded client for uptime_kuma.
pub struct UptimeKumaClient {
    http: HttpClient,
}

impl UptimeKumaClient {
    /// Build a scaffolded client.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UptimeKumaError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Placeholder health probe used by the scaffolded service.
    pub async fn health(&self) -> Result<(), UptimeKumaError> {
        let _ = &self.http;
        Ok(())
    }
}
