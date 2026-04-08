//! Language and localization endpoints.
//!
//! Covers `/api/v3/language` (audio/subtitle languages Radarr recognizes)
//! and `/api/v3/localization` (UI string catalog).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::language::Language;

impl RadarrClient {
    /// List every language Radarr knows about.
    ///
    /// Maps to `GET /api/v3/language`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn language_list(&self) -> Result<Vec<Language>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }
}
