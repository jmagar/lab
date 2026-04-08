//! Import-list endpoints.
//!
//! Covers `/api/v3/importlist`, `/api/v3/importlist/config`,
//! `/api/v3/importlistexclusion`, and `/api/v3/exclusions`.
//!
//! Import lists are Radarr's mechanism for automatically pulling movie
//! collections from upstream sources (TMDB, Trakt, Letterboxd, IMDb).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::import_list::{ImportList, ImportListExclusion};

impl RadarrClient {
    /// List every import list.
    ///
    /// Maps to `GET /api/v3/importlist`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn import_list_list(&self) -> Result<Vec<ImportList>, RadarrError> {
        self.http
            .get_json("/api/v3/importlist")
            .await
            .map_err(RadarrError::from)
    }

    /// List every import-list exclusion.
    ///
    /// Maps to `GET /api/v3/importlistexclusion`. Exclusions are TMDB ids
    /// that will never be added by any import list — used to keep unwanted
    /// movies out of an otherwise subscribed collection.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn import_list_exclusion_list(
        &self,
    ) -> Result<Vec<ImportListExclusion>, RadarrError> {
        self.http
            .get_json("/api/v3/importlistexclusion")
            .await
            .map_err(RadarrError::from)
    }
}
