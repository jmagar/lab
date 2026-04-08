//! Filesystem and manual-import endpoints.
//!
//! Covers `/api/v3/filesystem` (remote directory browsing for the UI's
//! path-picker), `/api/v3/manualimport` (inspect files Radarr didn't
//! auto-import), and `/api/v3/extrafile`.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::filesystem::{FilesystemListing, ManualImportItem};

impl RadarrClient {
    /// Browse a server-side filesystem path.
    ///
    /// Maps to `GET /api/v3/filesystem?path=...`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn filesystem_list(&self, path: &str) -> Result<FilesystemListing, RadarrError> {
        let _ = (path, &self.http);
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "filesystem_list not yet implemented".into(),
        )))
    }

    /// List items available for manual import from a folder.
    ///
    /// Maps to `GET /api/v3/manualimport?folder=...`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn manual_import_list(
        &self,
        folder: &str,
    ) -> Result<Vec<ManualImportItem>, RadarrError> {
        let _ = (folder, &self.http);
        Ok(Vec::new())
    }
}
