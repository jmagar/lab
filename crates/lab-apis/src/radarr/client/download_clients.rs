//! Download-client endpoints.
//!
//! Covers `/api/v3/downloadclient`, `/api/v3/downloadclient/config`, and
//! `/api/v3/remotepathmapping`.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::download_client::{DownloadClient, DownloadClientId, RemotePathMapping};

impl RadarrClient {
    /// List every configured download client (`SABnzbd`, `qBittorrent`, …).
    ///
    /// Maps to `GET /api/v3/downloadclient`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn download_client_list(&self) -> Result<Vec<DownloadClient>, RadarrError> {
        self.http
            .get_json("/api/v3/downloadclient")
            .await
            .map_err(RadarrError::from)
    }

    /// Test connectivity to a download client.
    ///
    /// Maps to `POST /api/v3/downloadclient/test`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` if the test fails or HTTP errors.
    pub async fn download_client_test(&self, id: DownloadClientId) -> Result<(), RadarrError> {
        let body = serde_json::json!({ "id": id.0 });
        self.http
            .post_void("/api/v3/downloadclient/test", &body)
            .await
            .map_err(RadarrError::from)
    }

    /// List every remote-path mapping.
    ///
    /// Maps to `GET /api/v3/remotepathmapping`. Path mappings translate
    /// download-client paths to Radarr-visible paths when the two don't
    /// share a filesystem view (Docker, remote downloaders).
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn remote_path_mapping_list(&self) -> Result<Vec<RemotePathMapping>, RadarrError> {
        self.http
            .get_json("/api/v3/remotepathmapping")
            .await
            .map_err(RadarrError::from)
    }
}
