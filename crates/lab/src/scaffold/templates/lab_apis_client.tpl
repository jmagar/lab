//! Scaffolded client for {{service}}.

use crate::core::{Auth, HttpClient};

use super::error::{{Service}}Error;

/// Scaffolded client for {{service}}.
pub struct {{Service}}Client {
    http: HttpClient,
}

impl {{Service}}Client {
    /// Build a scaffolded client.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, {{Service}}Error> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Placeholder health probe used by the scaffolded service.
    pub async fn health(&self) -> Result<(), {{Service}}Error> {
        let _ = &self.http;
        Ok(())
    }
}
