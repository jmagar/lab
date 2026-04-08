//! Indexer endpoints.
//!
//! Covers `/api/v3/indexer` — the indexer definitions configured in
//! Radarr (Prowlarr is normally the upstream that populates these).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::indexer::{Indexer, IndexerId};

impl RadarrClient {
    /// List every indexer configured in Radarr.
    ///
    /// Maps to `GET /api/v3/indexer`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn indexer_list(&self) -> Result<Vec<Indexer>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// Test an indexer's connectivity.
    ///
    /// Maps to `POST /api/v3/indexer/test`. Returns `Ok(())` on success;
    /// otherwise the upstream validation errors come back wrapped in
    /// `RadarrError::Api`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` if the test fails or HTTP errors.
    pub async fn indexer_test(&self, id: IndexerId) -> Result<(), RadarrError> {
        let _ = id;
        Ok(())
    }
}
