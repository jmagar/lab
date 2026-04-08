//! Metadata and media-cover endpoints.
//!
//! Covers `/api/v3/metadata` (Kodi/Plex/Emby metadata writers),
//! `/api/v3/metadata/config`, and `/api/v3/mediacover/{id}/{file}`
//! (poster/fanart asset serving).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::metadata::Metadata;

impl RadarrClient {
    /// List every metadata writer configuration.
    ///
    /// Maps to `GET /api/v3/metadata`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn metadata_list(&self) -> Result<Vec<Metadata>, RadarrError> {
        self.http
            .get_json("/api/v3/metadata")
            .await
            .map_err(RadarrError::from)
    }
}
