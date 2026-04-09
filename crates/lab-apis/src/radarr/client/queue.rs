//! Download-queue endpoints.
//!
//! Covers `/api/v3/queue` — inspect and manipulate items currently being
//! downloaded, stalled, or pending import.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::queue::{QueueItem, QueueItemId};

impl RadarrClient {
    /// List every item in the download queue, fetching all pages.
    ///
    /// Maps to `GET /api/v3/queue`. The response is paginated upstream;
    /// this method loops until all records are collected.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn queue_list(&self) -> Result<Vec<QueueItem>, RadarrError> {
        const PAGE_SIZE: i64 = 500;
        let mut all_records = Vec::new();
        let mut page = 1;

        loop {
            let val: serde_json::Value = self
                .http
                .get_json(&format!(
                    "/api/v3/queue?page={page}&pageSize={PAGE_SIZE}"
                ))
                .await
                .map_err(RadarrError::from)?;

            let records: Vec<QueueItem> = serde_json::from_value(
                val["records"].clone(),
            )
            .map_err(|e| RadarrError::Api(crate::core::error::ApiError::Decode(e.to_string())))?;

            let total = val["totalRecords"].as_i64().unwrap_or(0);
            all_records.extend(records);

            if (all_records.len() as i64) >= total {
                break;
            }
            page += 1;
        }

        Ok(all_records)
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
        self.http
            .delete(&format!(
                "/api/v3/queue/{}?removeFromClient={remove_from_client}&blocklist={blocklist}",
                id.0
            ))
            .await
            .map_err(RadarrError::from)
    }
}
