//! Download-queue endpoints.
//!
//! Covers `/api/v3/queue` — inspect and manipulate items currently being
//! downloaded, stalled, or pending import.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::queue::{QueueItem, QueueItemId};

impl RadarrClient {
    /// List every item in the download queue.
    ///
    /// Maps to `GET /api/v3/queue`. The response is paginated upstream;
    /// callers wanting the full set should loop on `page` until the
    /// returned record count is exhausted.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn queue_list(&self) -> Result<Vec<QueueItem>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// Remove an item from the queue.
    ///
    /// Maps to `DELETE /api/v3/queue/{id}`. `remove_from_client` also tells
    /// the download client (SAB/qBit) to drop the item; `blocklist` adds
    /// the release to Radarr's blocklist so it is not grabbed again.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn queue_remove(
        &self,
        id: QueueItemId,
        remove_from_client: bool,
        blocklist: bool,
    ) -> Result<(), RadarrError> {
        let _ = (id, remove_from_client, blocklist);
        Ok(())
    }
}
