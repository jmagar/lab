//! `ProwlarrClient` — indexer manager methods.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::ProwlarrError;
use super::types::HistoryQuery;

/// Client for a Prowlarr indexer manager instance.
pub struct ProwlarrClient {
    http: HttpClient,
}

impl ProwlarrClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Prowlarr uses header-token auth: pass
    /// `Auth::ApiKey { header: "X-Api-Key".into(), key: api_key }`.
    ///
    /// # Errors
    /// Returns [`ProwlarrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ProwlarrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Indexers ─────────────────────────────────────────────────────────────

    /// List all configured indexers.
    ///
    /// `GET /api/v1/indexer`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexers_list(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/indexer").await?)
    }

    /// Get a single indexer by ID.
    ///
    /// `GET /api/v1/indexer/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_get(&self, id: i64) -> Result<Value, ProwlarrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/indexer/{id}"))
            .await?)
    }

    /// Delete a single indexer by ID.
    ///
    /// `DELETE /api/v1/indexer/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_delete(&self, id: i64) -> Result<(), ProwlarrError> {
        Ok(self
            .http
            .delete(&format!("/api/v1/indexer/{id}"))
            .await?)
    }

    /// Test a single indexer.
    ///
    /// Prowlarr requires the full indexer resource object to be `POST`ed to
    /// `/api/v1/indexer/test`. This method first fetches the indexer by ID,
    /// then posts it to the test endpoint.
    ///
    /// `GET /api/v1/indexer/{id}` + `POST /api/v1/indexer/test`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_test(&self, id: i64) -> Result<Value, ProwlarrError> {
        let indexer: Value = self
            .http
            .get_json(&format!("/api/v1/indexer/{id}"))
            .await?;
        Ok(self
            .http
            .post_json("/api/v1/indexer/test", &indexer)
            .await?)
    }

    /// Test all configured indexers.
    ///
    /// `POST /api/v1/indexer/testall`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexers_testall(&self) -> Result<Value, ProwlarrError> {
        // testall has no request body — send an empty object.
        let empty: Value = serde_json::json!({});
        Ok(self
            .http
            .post_json("/api/v1/indexer/testall", &empty)
            .await?)
    }

    /// List indexer categories.
    ///
    /// `GET /api/v1/indexer/categories`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_categories(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/indexer/categories").await?)
    }

    // ── History ───────────────────────────────────────────────────────────────

    /// Get history with optional filters.
    ///
    /// `GET /api/v1/history`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn history_list(&self, query: &HistoryQuery) -> Result<Value, ProwlarrError> {
        let pairs = query.to_query_pairs();
        Ok(self
            .http
            .get_json_query("/api/v1/history", &pairs)
            .await?)
    }

    // ── Applications ──────────────────────────────────────────────────────────

    /// List configured applications (download clients connected to Prowlarr).
    ///
    /// `GET /api/v1/applications`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn applications_list(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/applications").await?)
    }

    /// Get a single application by ID.
    ///
    /// `GET /api/v1/applications/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn application_get(&self, id: i64) -> Result<Value, ProwlarrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/applications/{id}"))
            .await?)
    }

    /// Delete a single application by ID.
    ///
    /// `DELETE /api/v1/applications/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn application_delete(&self, id: i64) -> Result<(), ProwlarrError> {
        Ok(self
            .http
            .delete(&format!("/api/v1/applications/{id}"))
            .await?)
    }

    // ── System ────────────────────────────────────────────────────────────────

    /// Get system status (version, branch, OS, etc.).
    ///
    /// `GET /api/v1/system/status`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn system_status(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/system/status").await?)
    }

    /// Get system health alerts.
    ///
    /// `GET /api/v1/health`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn system_health(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/health").await?)
    }

    /// Health check — probe via system status (lightest authenticated endpoint).
    ///
    /// Calls `GET /api/v1/system/status` and discards the body. This is the
    /// fastest authenticated round-trip available in the Prowlarr API.
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn probe(&self) -> Result<(), ProwlarrError> {
        self.http.get_void("/api/v1/system/status").await?;
        Ok(())
    }
}
