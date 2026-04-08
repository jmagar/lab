//! `MemosClient` — memo management methods.
//!
//! TODO: implement memo CRUD, tag management, and resource uploads from the Memos REST API.

use crate::core::{Auth, HttpClient};

use super::error::MemosError;

/// Client for a Memos instance.
pub struct MemosClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl MemosClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Memos uses token auth: pass `Auth::Token { token: access_token }`.
    ///
    /// # Errors
    /// Returns [`MemosError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, MemosError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), MemosError> {
        // TODO: GET /api/v1/workspace/profile
        Ok(())
    }
}
