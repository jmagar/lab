//! `QbittorrentClient` — torrent management methods.
//!
//! qBittorrent's WebUI API v2 uses cookie-based session auth. Callers must
//! supply the SID cookie obtained from `POST /api/v2/auth/login`. The lab
//! binary handles login at client construction time by passing the credential
//! via `Auth::Session { cookie }`.

use crate::core::HttpClient;

use super::error::QbittorrentError;
use super::types::{
    Category, LogEntry, Preferences, Torrent, TorrentFile, TorrentProperties, Tracker, TransferInfo,
};

/// Client for a qBittorrent `WebUI` instance.
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

    /// Add one or more torrents by URL.
    ///
    /// `urls`: newline-separated magnet links or HTTP URLs.
    /// `savepath`, `category`, `tags`: optional metadata applied to the added torrent(s).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn add_torrent(
        &self,
        urls: &str,
        savepath: Option<&str>,
        category: Option<&str>,
        tags: Option<&str>,
    ) -> Result<(), QbittorrentError> {
        let mut fields: Vec<(&str, &str)> = vec![("urls", urls)];
        if let Some(sp) = savepath {
            fields.push(("savepath", sp));
        }
        if let Some(cat) = category {
            fields.push(("category", cat));
        }
        if let Some(t) = tags {
            fields.push(("tags", t));
        }
        // qBittorrent responds with "Ok." (200) or "Fails." (200 with error body).
        // post_form_void treats any 2xx as success; Fails. will surface as a 200 but
        // the body text is not checked here — callers inspect logs if needed.
        self.http
            .post_form_void("/api/v2/torrents/add", &fields)
            .await?;
        Ok(())
    }

    /// Toggle alternative speed limits mode (alt speeds vs. global limits).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn toggle_speed_limits(&self) -> Result<(), QbittorrentError> {
        self.http
            .post_form_void("/api/v2/transfer/toggleSpeedLimitsMode", &[])
            .await?;
        Ok(())
    }

    /// Get the file list for a specific torrent by hash.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn torrent_files(&self, hash: &str) -> Result<Vec<TorrentFile>, QbittorrentError> {
        let query = vec![("hash".to_string(), hash.to_string())];
        Ok(self
            .http
            .get_json_query("/api/v2/torrents/files", &query)
            .await?)
    }

    /// Set the priority for a specific file within a torrent.
    ///
    /// `id`: pipe-separated list of file indices.
    /// `priority`: 0 (do not download), 1 (normal), 6 (high), 7 (maximum).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_file_priority(
        &self,
        hash: &str,
        id: &str,
        priority: i32,
    ) -> Result<(), QbittorrentError> {
        let priority_s = priority.to_string();
        let fields: &[(&str, &str)] = &[("hash", hash), ("id", id), ("priority", &priority_s)];
        self.http
            .post_form_void("/api/v2/torrents/filePrio", fields)
            .await?;
        Ok(())
    }

    /// Move one or more torrents to a new save location.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_location(&self, hashes: &str, location: &str) -> Result<(), QbittorrentError> {
        let fields: &[(&str, &str)] = &[("hashes", hashes), ("location", location)];
        self.http
            .post_form_void("/api/v2/torrents/setLocation", fields)
            .await?;
        Ok(())
    }

    /// Add tags to one or more torrents.
    ///
    /// `hashes`: pipe-separated torrent hashes or `"all"`.
    /// `tags`: comma-separated tag names.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn add_tags(&self, hashes: &str, tags: &str) -> Result<(), QbittorrentError> {
        let fields: &[(&str, &str)] = &[("hashes", hashes), ("tags", tags)];
        self.http
            .post_form_void("/api/v2/torrents/addTags", fields)
            .await?;
        Ok(())
    }

    /// Remove tags from one or more torrents.
    ///
    /// `hashes`: pipe-separated torrent hashes or `"all"`.
    /// `tags`: comma-separated tag names (empty string removes all tags).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn remove_tags(&self, hashes: &str, tags: &str) -> Result<(), QbittorrentError> {
        let fields: &[(&str, &str)] = &[("hashes", hashes), ("tags", tags)];
        self.http
            .post_form_void("/api/v2/torrents/removeTags", fields)
            .await?;
        Ok(())
    }

    /// Force re-announce one or more torrents to all trackers.
    ///
    /// `hashes`: pipe-separated torrent hashes or `"all"`.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn reannounce(&self, hashes: &str) -> Result<(), QbittorrentError> {
        let fields: &[(&str, &str)] = &[("hashes", hashes)];
        self.http
            .post_form_void("/api/v2/torrents/reannounce", fields)
            .await?;
        Ok(())
    }

    /// Set share limits for one or more torrents.
    ///
    /// `hashes`: pipe-separated torrent hashes or `"all"`.
    /// `ratio_limit`: seeding ratio limit (-2 = use global, -1 = no limit, ≥0 = ratio cap).
    /// `seeding_time_limit`: seeding time limit in minutes (-2 = global, -1 = no limit, ≥0 = cap).
    /// `inactive_seeding_time_limit`: inactive seeding time limit in minutes.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn set_share_limits(
        &self,
        hashes: &str,
        ratio_limit: f64,
        seeding_time_limit: i64,
        inactive_seeding_time_limit: i64,
    ) -> Result<(), QbittorrentError> {
        let ratio_s = ratio_limit.to_string();
        let seeding_s = seeding_time_limit.to_string();
        let inactive_s = inactive_seeding_time_limit.to_string();
        let fields: &[(&str, &str)] = &[
            ("hashes", hashes),
            ("ratioLimit", &ratio_s),
            ("seedingTimeLimit", &seeding_s),
            ("inactiveSeedingTimeLimit", &inactive_s),
        ];
        self.http
            .post_form_void("/api/v2/torrents/setShareLimits", fields)
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

    /// Create a new torrent category.
    ///
    /// `category`: unique category name.
    /// `save_path`: optional default save path for this category.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn create_category(
        &self,
        category: &str,
        save_path: Option<&str>,
    ) -> Result<(), QbittorrentError> {
        let sp = save_path.unwrap_or("");
        let fields: &[(&str, &str)] = &[("category", category), ("savePath", sp)];
        self.http
            .post_form_void("/api/v2/torrents/createCategory", fields)
            .await?;
        Ok(())
    }

    /// Edit an existing torrent category.
    ///
    /// `category`: category name to edit.
    /// `save_path`: new default save path for this category (required).
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn edit_category(
        &self,
        category: &str,
        save_path: &str,
    ) -> Result<(), QbittorrentError> {
        let fields: &[(&str, &str)] = &[("category", category), ("savePath", save_path)];
        self.http
            .post_form_void("/api/v2/torrents/editCategory", fields)
            .await?;
        Ok(())
    }

    // ── Sync ─────────────────────────────────────────────────────────────────

    /// Get sync maindata — full state dump or incremental delta.
    ///
    /// `rid`: response ID from the previous call. Pass `0` for the first call
    /// (triggers a full dump). Pass the `rid` returned in the previous response
    /// on subsequent calls to receive only changes since then.
    ///
    /// # Errors
    /// Returns `QbittorrentError::Api` on HTTP failure.
    pub async fn sync_maindata(&self, rid: i64) -> Result<serde_json::Value, QbittorrentError> {
        let rid_s = rid.to_string();
        let query = vec![("rid".to_string(), rid_s)];
        Ok(self
            .http
            .get_json_query("/api/v2/sync/maindata", &query)
            .await?)
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
