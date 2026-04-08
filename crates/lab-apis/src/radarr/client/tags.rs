//! Tag endpoints.
//!
//! Covers `/api/v3/tag` and `/api/v3/tag/detail` — free-form labels
//! attached to movies, indexers, download clients, notifications, etc.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::tag::{Tag, TagDetail};

impl RadarrClient {
    /// List every tag.
    ///
    /// Maps to `GET /api/v3/tag`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn tag_list(&self) -> Result<Vec<Tag>, RadarrError> {
        self.http
            .get_json("/api/v3/tag")
            .await
            .map_err(RadarrError::from)
    }

    /// List tags with usage detail (which resources carry them).
    ///
    /// Maps to `GET /api/v3/tag/detail`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn tag_detail_list(&self) -> Result<Vec<TagDetail>, RadarrError> {
        self.http
            .get_json("/api/v3/tag/detail")
            .await
            .map_err(RadarrError::from)
    }
}
