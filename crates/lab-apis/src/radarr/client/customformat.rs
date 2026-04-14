//! Custom format endpoints.
//!
//! Covers `/api/v3/customformat` — quality scoring rules that let Radarr
//! prefer releases with certain properties (codec, HDR, source, etc.).

use super::RadarrClient;
use crate::radarr::error::RadarrError;

impl RadarrClient {
    /// List all custom formats.
    ///
    /// Maps to `GET /api/v3/customformat`. Returns the full list of custom
    /// format definitions configured in Radarr.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn customformat_list(&self) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json("/api/v3/customformat")
            .await
            .map_err(RadarrError::from)
    }
}
