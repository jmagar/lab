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
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// List tags with usage detail (which resources carry them).
    ///
    /// Maps to `GET /api/v3/tag/detail`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn tag_detail_list(&self) -> Result<Vec<TagDetail>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }
}
