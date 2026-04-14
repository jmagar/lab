//! Wanted endpoints.
//!
//! Covers `/api/v3/wanted/missing` (movies monitored but not downloaded) and
//! `/api/v3/wanted/cutoff` (movies that do not meet quality cutoff).

use super::RadarrClient;
use crate::radarr::error::RadarrError;

impl RadarrClient {
    /// List movies that are monitored but missing from the library.
    ///
    /// Maps to `GET /api/v3/wanted/missing`. Supports optional pagination via
    /// `page` and `page_size` query parameters.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn wanted_missing(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json(&format!(
                "/api/v3/wanted/missing?page={page}&pageSize={page_size}"
            ))
            .await
            .map_err(RadarrError::from)
    }

    /// List movies that are below the quality cutoff.
    ///
    /// Maps to `GET /api/v3/wanted/cutoff`. Supports optional pagination via
    /// `page` and `page_size` query parameters.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn wanted_cutoff(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json(&format!(
                "/api/v3/wanted/cutoff?page={page}&pageSize={page_size}"
            ))
            .await
            .map_err(RadarrError::from)
    }
}
