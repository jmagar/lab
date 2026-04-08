//! Config endpoints.
//!
//! Covers the sprawling `/api/v3/config/*` surface — host, naming, UI,
//! media management, download client, indexer, and import list config —
//! plus `/api/v3/autotagging`.
//!
//! This file is expected to grow large; when it does, nest as
//! `client/config/{host,naming,ui,media_management,…}.rs` — the public
//! method surface on `RadarrClient` stays flat.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::config::{HostConfig, NamingConfig, UiConfig};

impl RadarrClient {
    /// Fetch the host config (URL base, bind address, auth settings, …).
    ///
    /// Maps to `GET /api/v3/config/host`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn host_config_get(&self) -> Result<HostConfig, RadarrError> {
        let _ = &self.http;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "host_config_get not yet implemented".into(),
        )))
    }

    /// Fetch the naming config (movie filename format, folder format, …).
    ///
    /// Maps to `GET /api/v3/config/naming`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn naming_config_get(&self) -> Result<NamingConfig, RadarrError> {
        let _ = &self.http;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "naming_config_get not yet implemented".into(),
        )))
    }

    /// Fetch the UI config (theme, date format, …).
    ///
    /// Maps to `GET /api/v3/config/ui`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn ui_config_get(&self) -> Result<UiConfig, RadarrError> {
        let _ = &self.http;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "ui_config_get not yet implemented".into(),
        )))
    }
}
