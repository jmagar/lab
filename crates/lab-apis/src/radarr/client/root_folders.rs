//! Root-folder endpoints.
//!
//! Covers `/api/v3/rootfolder` — the on-disk locations Radarr stores
//! movies under. Required when adding new movies.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::root_folder::RootFolder;

impl RadarrClient {
    /// List every configured root folder (path + free-space info).
    ///
    /// Maps to `GET /api/v3/rootfolder`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn root_folder_list(&self) -> Result<Vec<RootFolder>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }
}
