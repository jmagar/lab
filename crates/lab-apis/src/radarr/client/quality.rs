//! Quality endpoints.
//!
//! Covers `/api/v3/qualityprofile` and `/api/v3/qualitydefinition`.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::quality::{QualityDefinition, QualityProfile};

impl RadarrClient {
    /// List every quality profile.
    ///
    /// Maps to `GET /api/v3/qualityprofile`. Profiles drive what qualities
    /// Radarr will accept for a given movie and the upgrade behavior.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn quality_profile_list(&self) -> Result<Vec<QualityProfile>, RadarrError> {
        self.http
            .get_json("/api/v3/qualityprofile")
            .await
            .map_err(RadarrError::from)
    }

    /// List every quality definition (the size/megabit rules Radarr uses
    /// per quality level).
    ///
    /// Maps to `GET /api/v3/qualitydefinition`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn quality_definition_list(&self) -> Result<Vec<QualityDefinition>, RadarrError> {
        self.http
            .get_json("/api/v3/qualitydefinition")
            .await
            .map_err(RadarrError::from)
    }
}
