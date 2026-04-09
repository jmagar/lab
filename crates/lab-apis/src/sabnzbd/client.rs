//! `SabnzbdClient` — Usenet download management methods.
//!
//! SABnzbd uses a query-parameter-based API: `GET /api?mode=<action>&apikey=<key>&output=json`.
//! The API key is passed as a query param, not a header, so the client stores
//! it separately and uses `Auth::None` for the underlying `HttpClient`.

use crate::core::HttpClient;

use super::error::SabnzbdError;
use super::types::{
    HistoryResponse, QueueResponse, ServerStats, StatusResponse, VersionResponse,
    WarningsResponse,
};

/// Client for a SABnzbd instance.
pub struct SabnzbdClient {
    http: HttpClient,
    api_key: String,
}

impl SabnzbdClient {
    /// Build a client against `base_url` with an API key.
    ///
    /// SABnzbd authenticates via query parameter, not header. Pass the raw
    /// API key; the client appends `&apikey=<key>` to every request.
    ///
    /// # Errors
    /// Returns [`SabnzbdError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, api_key: String) -> Result<Self, SabnzbdError> {
        Ok(Self {
            http: HttpClient::new(base_url, crate::core::Auth::None)?,
            api_key,
        })
    }

    /// Build the common query params for every SABnzbd API call.
    fn base_query(&self, mode: &str) -> Vec<(String, String)> {
        vec![
            ("mode".to_string(), mode.to_string()),
            ("apikey".to_string(), self.api_key.clone()),
            ("output".to_string(), "json".to_string()),
        ]
    }

    /// Get the SABnzbd version string.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn version(&self) -> Result<String, SabnzbdError> {
        let resp: VersionResponse = self
            .http
            .get_json_query("/api", &self.base_query("version"))
            .await?;
        Ok(resp.version)
    }

    /// Get the current download queue.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn queue(&self) -> Result<QueueResponse, SabnzbdError> {
        let resp: QueueResponse = self
            .http
            .get_json_query("/api", &self.base_query("queue"))
            .await?;
        Ok(resp)
    }

    /// Get download history.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn history(&self, limit: Option<u32>) -> Result<HistoryResponse, SabnzbdError> {
        let mut query = self.base_query("history");
        if let Some(n) = limit {
            query.push(("limit".to_string(), n.to_string()));
        }
        let resp: HistoryResponse = self.http.get_json_query("/api", &query).await?;
        Ok(resp)
    }

    /// Get server stats (total/month/week/day byte counts).
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn server_stats(&self) -> Result<ServerStats, SabnzbdError> {
        let resp: ServerStats = self
            .http
            .get_json_query("/api", &self.base_query("server_stats"))
            .await?;
        Ok(resp)
    }

    /// Get warnings.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn warnings(&self) -> Result<Vec<String>, SabnzbdError> {
        let resp: WarningsResponse = self
            .http
            .get_json_query("/api", &self.base_query("warnings"))
            .await?;
        Ok(resp.warnings)
    }

    /// Pause the download queue.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn pause(&self) -> Result<bool, SabnzbdError> {
        let resp: StatusResponse = self
            .http
            .get_json_query("/api", &self.base_query("pause"))
            .await?;
        Ok(resp.status)
    }

    /// Resume the download queue.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn resume(&self) -> Result<bool, SabnzbdError> {
        let resp: StatusResponse = self
            .http
            .get_json_query("/api", &self.base_query("resume"))
            .await?;
        Ok(resp.status)
    }

    /// Set the download speed limit (KB/s). Pass 0 to remove the limit.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn set_speed_limit(&self, kbps: u64) -> Result<bool, SabnzbdError> {
        let mut query = self.base_query("config");
        query.push(("name".to_string(), "speedlimit".to_string()));
        query.push(("value".to_string(), kbps.to_string()));
        let resp: StatusResponse = self.http.get_json_query("/api", &query).await?;
        Ok(resp.status)
    }

    /// Delete a queue item by `nzo_id`.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn queue_delete(&self, nzo_id: &str) -> Result<bool, SabnzbdError> {
        let mut query = self.base_query("queue");
        query.push(("name".to_string(), "delete".to_string()));
        query.push(("value".to_string(), nzo_id.to_string()));
        let resp: StatusResponse = self.http.get_json_query("/api", &query).await?;
        Ok(resp.status)
    }

    /// Delete a history item by `nzo_id`.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn history_delete(&self, nzo_id: &str) -> Result<bool, SabnzbdError> {
        let mut query = self.base_query("history");
        query.push(("name".to_string(), "delete".to_string()));
        query.push(("value".to_string(), nzo_id.to_string()));
        let resp: StatusResponse = self.http.get_json_query("/api", &query).await?;
        Ok(resp.status)
    }

    /// Purge all completed history items.
    ///
    /// # Errors
    /// Returns `SabnzbdError::Api` on HTTP failure.
    pub async fn history_purge(&self) -> Result<bool, SabnzbdError> {
        let mut query = self.base_query("history");
        query.push(("name".to_string(), "delete".to_string()));
        query.push(("value".to_string(), "all".to_string()));
        let resp: StatusResponse = self.http.get_json_query("/api", &query).await?;
        Ok(resp.status)
    }
}
