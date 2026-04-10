//! History and blocklist endpoints.
//!
//! Covers `/api/v3/history` (grab/import/upgrade/deletion records) and
//! `/api/v3/blocklist` (releases Radarr has been told to skip).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::history::{BlocklistItem, HistoryPage};

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
            .map(serde_json::Value::take)
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(records)
            .map_err(|e| RadarrError::Api(crate::core::error::ApiError::Decode(e.to_string())))
    }
}
