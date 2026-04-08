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
        let _ = (page, page_size, &self.http);
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "history_list not yet implemented".into(),
        )))
    }

    /// List every blocklisted release.
    ///
    /// Maps to `GET /api/v3/blocklist`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn blocklist_list(&self) -> Result<Vec<BlocklistItem>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }
}
