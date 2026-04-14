//! History and blocklist endpoints.
//!
//! Covers `/api/v3/history` (grab/import/upgrade/deletion records),
//! `/api/v3/blocklist` (releases Radarr has been told to skip), and
//! `/api/v3/history/failed/{id}` (mark a history record as failed and retry).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::history::{BlocklistId, BlocklistItem, HistoryId, HistoryPage};
use crate::radarr::types::movie::MovieId;

impl RadarrClient {
    /// Fetch a page of history records.
    ///
    /// Maps to `GET /api/v3/history`. Radarr paginates server-side;
    /// callers needing the full set should loop on `page`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn history_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<HistoryPage, RadarrError> {
        self.http
            .get_json(&format!("/api/v3/history?page={page}&pageSize={page_size}"))
            .await
            .map_err(RadarrError::from)
    }

    /// List every blocklisted release.
    ///
    /// Maps to `GET /api/v3/blocklist`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn blocklist_list(&self) -> Result<Vec<BlocklistItem>, RadarrError> {
        let mut val: serde_json::Value = self
            .http
            .get_json("/api/v3/blocklist?pageSize=1000")
            .await
            .map_err(RadarrError::from)?;
        let records = val
            .get_mut("records")
            .map_or(serde_json::Value::Null, serde_json::Value::take);
        serde_json::from_value(records)
            .map_err(|e| RadarrError::Api(crate::core::error::ApiError::Decode(e.to_string())))
    }

    /// Delete a specific blocklist entry.
    ///
    /// Maps to `DELETE /api/v3/blocklist/{id}`. Destructive — callers should
    /// confirm at the CLI/MCP boundary.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn blocklist_delete(&self, id: BlocklistId) -> Result<(), RadarrError> {
        self.http
            .delete(&format!("/api/v3/blocklist/{}", id.0))
            .await
            .map_err(RadarrError::from)
    }

    /// List history records for a specific movie.
    ///
    /// Maps to `GET /api/v3/history/movie?movieId={id}`. Returns all history
    /// events for that movie (grabs, imports, failures, etc.).
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn history_movie(&self, movie_id: MovieId) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json(&format!("/api/v3/history/movie?movieId={}", movie_id.0))
            .await
            .map_err(RadarrError::from)
    }

    /// Mark a history record as failed and trigger a retry.
    ///
    /// Maps to `POST /api/v3/history/failed/{id}`. Radarr marks the given
    /// history record as failed and immediately queues a new search for
    /// the associated movie.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn history_failed_retry(&self, id: HistoryId) -> Result<(), RadarrError> {
        self.http
            .post_void(
                &format!("/api/v3/history/failed/{}", id.0),
                &serde_json::Value::Null,
            )
            .await
            .map_err(RadarrError::from)
    }
}
