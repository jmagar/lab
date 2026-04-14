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
        Ok(self.http.get_json(&format!("/api/v1/indexer/{id}")).await?)
    }

    /// Delete a single indexer by ID.
    ///
    /// `DELETE /api/v1/indexer/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_delete(&self, id: i64) -> Result<(), ProwlarrError> {
        Ok(self.http.delete(&format!("/api/v1/indexer/{id}")).await?)
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
        let indexer: Value = self.http.get_json(&format!("/api/v1/indexer/{id}")).await?;
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
        Ok(self.http.get_json_query("/api/v1/history", &pairs).await?)
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

    /// Edit (update) an indexer by ID.
    ///
    /// `PUT /api/v1/indexer/{id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_edit(&self, id: i64, body: &Value) -> Result<Value, ProwlarrError> {
        Ok(self
            .http
            .put_json(&format!("/api/v1/indexer/{id}"), body)
            .await?)
    }

    /// Add (create) a new indexer.
    ///
    /// `POST /api/v1/indexer`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_add(&self, body: &Value) -> Result<Value, ProwlarrError> {
        Ok(self.http.post_json("/api/v1/indexer", body).await?)
    }

    /// Get indexer statistics.
    ///
    /// `GET /api/v1/indexerstats`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_stats(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/indexerstats").await?)
    }

    /// Get indexer status (indexers with errors).
    ///
    /// `GET /api/v1/indexerstatus`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_status(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/indexerstatus").await?)
    }

    /// Search across indexers.
    ///
    /// `GET /api/v1/search?query=...&indexerIds[]=...&categories[]=...&type=search`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_search(
        &self,
        query: &str,
        indexer_ids: &[i64],
        categories: &[i64],
        search_type: Option<&str>,
    ) -> Result<Value, ProwlarrError> {
        let mut pairs: Vec<(String, String)> = Vec::new();
        pairs.push(("query".to_string(), query.to_string()));
        for id in indexer_ids {
            pairs.push(("indexerIds[]".to_string(), id.to_string()));
        }
        for cat in categories {
            pairs.push(("categories[]".to_string(), cat.to_string()));
        }
        if let Some(t) = search_type {
            pairs.push(("type".to_string(), t.to_string()));
        } else {
            pairs.push(("type".to_string(), "search".to_string()));
        }
        Ok(self.http.get_json_query("/api/v1/search", &pairs).await?)
    }

    /// Grab a release (send to download client) by GUID.
    ///
    /// `POST /api/v1/search` with body `{"guid": "..."}`.
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn indexer_grab(&self, guid: &str) -> Result<Value, ProwlarrError> {
        let body = serde_json::json!({ "guid": guid });
        Ok(self.http.post_json("/api/v1/search", &body).await?)
    }

    /// Get history for a specific indexer.
    ///
    /// `GET /api/v1/history?indexerId={id}`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn history_indexer(&self, indexer_id: i64) -> Result<Value, ProwlarrError> {
        let pairs = vec![("indexerId".to_string(), indexer_id.to_string())];
        Ok(self.http.get_json_query("/api/v1/history", &pairs).await?)
    }

    /// Add (create) a new application.
    ///
    /// `POST /api/v1/applications`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn application_add(&self, body: &Value) -> Result<Value, ProwlarrError> {
        Ok(self.http.post_json("/api/v1/applications", body).await?)
    }

    /// Restart the Prowlarr server.
    ///
    /// `POST /api/v1/system/restart`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn system_restart(&self) -> Result<(), ProwlarrError> {
        let empty = serde_json::json!({});
        Ok(self
            .http
            .post_void("/api/v1/system/restart", &empty)
            .await?)
    }

    /// Get system backups.
    ///
    /// `GET /api/v1/system/backup`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn system_backup(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/system/backup").await?)
    }

    /// List all tags.
    ///
    /// `GET /api/v1/tag`
    ///
    /// # Errors
    /// Returns `ProwlarrError::Api` on HTTP failure.
    pub async fn tag_list(&self) -> Result<Value, ProwlarrError> {
        Ok(self.http.get_json("/api/v1/tag").await?)
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
