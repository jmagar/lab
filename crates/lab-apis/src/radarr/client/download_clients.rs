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

    /// Get a single download client by ID.
    ///
    /// Maps to `GET /api/v3/downloadclient/{id}`.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` when the ID is unknown; `RadarrError::Api` on other HTTP failures.
    pub async fn download_client_get(
        &self,
        id: DownloadClientId,
    ) -> Result<DownloadClient, RadarrError> {
        use crate::core::ApiError;
        self.http
            .get_json(&format!("/api/v3/downloadclient/{}", id.0))
            .await
            .map_err(|e| match e {
                ApiError::NotFound => RadarrError::NotFound {
                    kind: "download_client",
                    id: id.0,
                },
                other => RadarrError::Api(other),
            })
    }

    /// Test connectivity to a download client.
    ///
    /// Maps to `POST /api/v3/downloadclient/test`. The Radarr API requires the
    /// full download client resource as the request body — not just the id.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` if the test fails or HTTP errors.
    pub async fn download_client_test(&self, client: &DownloadClient) -> Result<(), RadarrError> {
        self.http
            .post_void("/api/v3/downloadclient/test", client)
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
