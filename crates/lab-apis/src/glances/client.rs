//! Scaffolded client for glances.

use crate::core::{Auth, HttpClient};

use super::error::GlancesError;

/// Scaffolded client for glances.
pub struct GlancesClient {
    http: HttpClient,
}

impl GlancesClient {
    /// Build a scaffolded client.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, GlancesError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Placeholder health probe used by the scaffolded service.
    pub async fn health(&self) -> Result<(), GlancesError> {
        let _ = &self.http;
        Ok(())
    }
}
