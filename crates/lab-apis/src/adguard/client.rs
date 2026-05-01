//! Scaffolded client for adguard.

use crate::core::{Auth, HttpClient};

use super::error::AdguardError;

/// Scaffolded client for adguard.
pub struct AdguardClient {
    http: HttpClient,
}

impl AdguardClient {
    /// Build a scaffolded client.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, AdguardError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Placeholder health probe used by the scaffolded service.
    pub async fn health(&self) -> Result<(), AdguardError> {
        let _ = &self.http;
        Ok(())
    }
}
