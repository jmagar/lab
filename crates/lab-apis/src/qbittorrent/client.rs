//! `QbittorrentClient` — torrent management methods.
//!
//! qBittorrent's WebUI API v2 uses cookie-based session auth. Callers must
//! supply the SID cookie obtained from `POST /api/v2/auth/login`. The lab
//! binary handles login at client construction time by passing the credential
//! via `Auth::Session { cookie }`.

use crate::core::HttpClient;

use super::error::QbittorrentError;
use super::types::{
    Category, LogEntry, Preferences, Torrent, TorrentProperties, Tracker, TransferInfo,
};

/// Client for a qBittorrent WebUI instance.
pub struct QbittorrentClient {
    http: HttpClient,
}

impl QbittorrentClient {
    /// Build a client against `base_url` with a session cookie (`SID=<value>`).
    ///
    /// The caller is responsible for obtaining the SID cookie by calling
    /// `POST /api/v2/auth/login` externally. Pass the full `Cookie` header value,
    /// e.g. `"SID=abc123"`.
    ///
    /// Use `Auth::Session` to inject the cookie on every request.
    ///
    /// # Errors
    /// Returns [`QbittorrentError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, sid_cookie: String) -> Result<Self, QbittorrentError> {
        Ok(Self {
            http: HttpClient::new(base_url, crate::core::Auth::Session { cookie: sid_cookie })?,
        })
    }

    /// Build a single-pair query string from a key/value pair.
    fn q1(k: &str, v: impl Into<String>) -> Vec<(String, String)> {
        vec![(k.to_string(), v.into())]
    }

    // ── Application ──────────────────────────────────────────────────────────

    /// Get the qBittorrent application version string.
    ///
    /// The `/api/v2/app/version` endpoint returns a plain text body (e.g. `v4.6.3`).
    /// We parse it as a JSON `Value` to be tolerant of both plain-text and
    /// JSON-quoted responses, then extract the string.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn version(&self) -> Result<String, QbittorrentError> {
        let raw: serde_json::Value = self.http.get_json("/api/v2/app/version").await?;
        let v = match raw {
            serde_json::Value::String(s) => s,
            other => other.to_string(),
        };
        Ok(v)
    }

    /// Get qBittorrent application preferences.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn preferences(&self) -> Result<Preferences, QbittorrentError> {
        Ok(self.http.get_json("/api/v2/app/preferences").await?)
    }

    // ── Transfer ─────────────────────────────────────────────────────────────

    /// Get global transfer info (speeds, limits, DHT nodes, connection status).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn transfer_info(&self) -> Result<TransferInfo, QbittorrentError> {
        Ok(self.http.get_json("/api/v2/transfer/info").await?)
    }

    /// Set global download speed limit in bytes/s (0 = unlimited).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_download_limit(&self, limit: i64) -> Result<(), QbittorrentError> {
        let query = Self::q1("limit", limit.to_string());
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/transfer/setDownloadLimit", &query)
            .await?;
        Ok(())
    }

    /// Set global upload speed limit in bytes/s (0 = unlimited).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_upload_limit(&self, limit: i64) -> Result<(), QbittorrentError> {
        let query = Self::q1("limit", limit.to_string());
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/transfer/setUploadLimit", &query)
            .await?;
        Ok(())
    }

    // ── Torrents ─────────────────────────────────────────────────────────────

    /// List all torrents, optionally filtered by status or category.
    ///
    /// `filter`: `all` | `downloading` | `seeding` | `completed` | `paused` |
    ///           `active` | `inactive` | `resumed` | `stalled` | `stalled_uploading` |
    ///           `stalled_downloading` | `errored`
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn list_torrents(
        &self,
        filter: Option<&str>,
        category: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<Torrent>, QbittorrentError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(f) = filter {
            query.push(("filter".to_string(), f.to_string()));
        }
        if let Some(c) = category {
            query.push(("category".to_string(), c.to_string()));
        }
        if let Some(n) = limit {
            query.push(("limit".to_string(), n.to_string()));
        }
        Ok(self
            .http
            .get_json_query("/api/v2/torrents/info", &query)
            .await?)
    }

    /// Get properties of a specific torrent by hash.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn torrent_properties(
        &self,
        hash: &str,
    ) -> Result<TorrentProperties, QbittorrentError> {
        let query = vec![("hash".to_string(), hash.to_string())];
        Ok(self
            .http
            .get_json_query("/api/v2/torrents/properties", &query)
            .await?)
    }

    /// Get trackers for a specific torrent by hash.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn torrent_trackers(&self, hash: &str) -> Result<Vec<Tracker>, QbittorrentError> {
        let query = vec![("hash".to_string(), hash.to_string())];
        Ok(self
            .http
            .get_json_query("/api/v2/torrents/trackers", &query)
            .await?)
    }

    /// Pause one or more torrents by hash (pipe-separated for multiple).
    ///
    /// Pass `"all"` to pause every torrent.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn pause_torrents(&self, hashes: &str) -> Result<(), QbittorrentError> {
        let query = Self::q1("hashes", hashes);
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/pause", &query)
            .await?;
        Ok(())
    }

    /// Resume one or more torrents by hash (pipe-separated for multiple).
    ///
    /// Pass `"all"` to resume every torrent.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn resume_torrents(&self, hashes: &str) -> Result<(), QbittorrentError> {
        let query = Self::q1("hashes", hashes);
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/resume", &query)
            .await?;
        Ok(())
    }

    /// Delete one or more torrents by hash (pipe-separated for multiple).
    ///
    /// Pass `"all"` to delete every torrent.
    /// Set `delete_files` to also remove the downloaded data.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn delete_torrents(
        &self,
        hashes: &str,
        delete_files: bool,
    ) -> Result<(), QbittorrentError> {
        let query = vec![
            ("hashes".to_string(), hashes.to_string()),
            ("deleteFiles".to_string(), delete_files.to_string()),
        ];
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/delete", &query)
            .await?;
        Ok(())
    }

    /// Set the download limit for one or more torrents (bytes/s; 0 = unlimited).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_torrent_download_limit(
        &self,
        hashes: &str,
        limit: i64,
    ) -> Result<(), QbittorrentError> {
        let query = vec![
            ("hashes".to_string(), hashes.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/setDownloadLimit", &query)
            .await?;
        Ok(())
    }

    /// Set the upload limit for one or more torrents (bytes/s; 0 = unlimited).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_torrent_upload_limit(
        &self,
        hashes: &str,
        limit: i64,
    ) -> Result<(), QbittorrentError> {
        let query = vec![
            ("hashes".to_string(), hashes.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/setUploadLimit", &query)
            .await?;
        Ok(())
    }

    /// Recheck one or more torrents (re-hash verification).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn recheck_torrents(&self, hashes: &str) -> Result<(), QbittorrentError> {
        let query = Self::q1("hashes", hashes);
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/recheck", &query)
            .await?;
        Ok(())
    }

    /// Set the category for one or more torrents.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_category(&self, hashes: &str, category: &str) -> Result<(), QbittorrentError> {
        let query = vec![
            ("hashes".to_string(), hashes.to_string()),
            ("category".to_string(), category.to_string()),
        ];
        self.http
            .get_json_query::<serde_json::Value>("/api/v2/torrents/setCategory", &query)
            .await?;
        Ok(())
    }

    // ── Categories ───────────────────────────────────────────────────────────

    /// List all categories.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn categories(
        &self,
    ) -> Result<std::collections::HashMap<String, Category>, QbittorrentError> {
        Ok(self.http.get_json("/api/v2/torrents/categories").await?)
    }

    // ── Log ──────────────────────────────────────────────────────────────────

    /// Get application log entries.
    ///
    /// `last_known_id`: return only entries after this ID (-1 = all).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn log(&self, last_known_id: Option<i64>) -> Result<Vec<LogEntry>, QbittorrentError> {
        let id = last_known_id.unwrap_or(-1);
        let query = vec![
            ("normal".to_string(), "true".to_string()),
            ("info".to_string(), "true".to_string()),
            ("warning".to_string(), "true".to_string()),
            ("critical".to_string(), "true".to_string()),
            ("last_known_id".to_string(), id.to_string()),
        ];
        Ok(self.http.get_json_query("/api/v2/log/main", &query).await?)
    }
}
