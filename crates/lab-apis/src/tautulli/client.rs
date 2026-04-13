//! `TautulliClient` — Tautulli Plex analytics methods.
//!
//! Tautulli authenticates via an `apikey` query parameter on every request.
//! The client stores the key and appends it to all outgoing requests.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::TautulliError;
use super::types::HistoryQuery;

/// Client for a Tautulli instance.
pub struct TautulliClient {
    http: HttpClient,
    api_key: String,
}

impl TautulliClient {
    /// Build a client against `base_url` with an API key.
    ///
    /// Tautulli uses `?apikey=<key>` query param auth — the key is stored
    /// and appended to every request in the query string.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, api_key: String) -> Result<Self, TautulliError> {
        Ok(Self {
            http: HttpClient::new(base_url, Auth::None)?,
            api_key,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Build the base query params including the API key and command.
    fn base_params(&self, cmd: &str) -> Vec<(String, String)> {
        vec![
            ("apikey".to_string(), self.api_key.clone()),
            ("cmd".to_string(), cmd.to_string()),
        ]
    }

    /// Run a Tautulli API command and return the raw JSON value.
    async fn api_get(&self, cmd: &str, extra: &[(&str, String)]) -> Result<Value, TautulliError> {
        let mut params = self.base_params(cmd);
        for (k, v) in extra {
            params.push(((*k).to_string(), v.clone()));
        }
        let raw: Value = self.http.get_json_query("/api/v2", &params).await?;
        // Tautulli wraps everything in {"response": {"result": "success", "data": ...}}
        // We return the inner data if available, otherwise the full response.
        if let Some(data) = raw
            .get("response")
            .and_then(|r| r.get("data"))
        {
            Ok(data.clone())
        } else {
            Ok(raw)
        }
    }

    // ── Activity ──────────────────────────────────────────────────────────────

    /// Get current Plex activity (active sessions).
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_activity(&self) -> Result<Value, TautulliError> {
        self.api_get("get_activity", &[]).await
    }

    /// Get details for a single active session by session key.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_stream_data(&self, session_key: &str) -> Result<Value, TautulliError> {
        self.api_get("get_stream_data", &[("session_key", session_key.to_string())])
            .await
    }

    // ── History ───────────────────────────────────────────────────────────────

    /// Get play history with optional filters.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_history(&self, query: &HistoryQuery) -> Result<Value, TautulliError> {
        let mut extra: Vec<(&str, String)> = Vec::new();
        if let Some(p) = query.page {
            extra.push(("start", ((p.saturating_sub(1)) * query.page_size.unwrap_or(25)).to_string()));
        }
        if let Some(ps) = query.page_size {
            extra.push(("length", ps.to_string()));
        }
        if let Some(ref dir) = query.order_dir {
            extra.push(("order_dir", dir.clone()));
        }
        if let Some(ref mt) = query.media_type {
            extra.push(("media_type", mt.clone()));
        }
        if let Some(uid) = query.user_id {
            extra.push(("user_id", uid.to_string()));
        }
        if let Some(sid) = query.section_id {
            extra.push(("section_id", sid.to_string()));
        }
        if let Some(rk) = query.rating_key {
            extra.push(("rating_key", rk.to_string()));
        }
        self.api_get("get_history", &extra).await
    }

    // ── Users ─────────────────────────────────────────────────────────────────

    /// Get all Tautulli users.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_users(&self) -> Result<Value, TautulliError> {
        self.api_get("get_users", &[]).await
    }

    /// Get user details by user ID.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_user(&self, user_id: i64) -> Result<Value, TautulliError> {
        self.api_get("get_user", &[("user_id", user_id.to_string())])
            .await
    }

    /// Get watch time statistics for a user.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_user_watch_time_stats(&self, user_id: i64) -> Result<Value, TautulliError> {
        self.api_get(
            "get_user_watch_time_stats",
            &[("user_id", user_id.to_string())],
        )
        .await
    }

    /// Get player statistics for a user.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_user_player_stats(&self, user_id: i64) -> Result<Value, TautulliError> {
        self.api_get(
            "get_user_player_stats",
            &[("user_id", user_id.to_string())],
        )
        .await
    }

    // ── Libraries ─────────────────────────────────────────────────────────────

    /// Get all Plex libraries tracked by Tautulli.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_libraries(&self) -> Result<Value, TautulliError> {
        self.api_get("get_libraries", &[]).await
    }

    /// Get library details by section ID.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_library(&self, section_id: i64) -> Result<Value, TautulliError> {
        self.api_get("get_library", &[("section_id", section_id.to_string())])
            .await
    }

    /// Get media info for a library.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_library_media_info(&self, section_id: i64) -> Result<Value, TautulliError> {
        self.api_get(
            "get_library_media_info",
            &[("section_id", section_id.to_string())],
        )
        .await
    }

    // ── Statistics ────────────────────────────────────────────────────────────

    /// Get home stats (most played, most active users, etc.).
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_home_stats(
        &self,
        time_range: Option<u32>,
        stats_count: Option<u32>,
    ) -> Result<Value, TautulliError> {
        let mut extra: Vec<(&str, String)> = Vec::new();
        if let Some(t) = time_range {
            extra.push(("time_range", t.to_string()));
        }
        if let Some(c) = stats_count {
            extra.push(("stats_count", c.to_string()));
        }
        self.api_get("get_home_stats", &extra).await
    }

    /// Get play count statistics by time interval.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_plays_by_date(
        &self,
        time_range: Option<u32>,
        y_axis: Option<&str>,
    ) -> Result<Value, TautulliError> {
        let mut extra: Vec<(&str, String)> = Vec::new();
        if let Some(t) = time_range {
            extra.push(("time_range", t.to_string()));
        }
        if let Some(y) = y_axis {
            extra.push(("y_axis", y.to_string()));
        }
        self.api_get("get_plays_by_date", &extra).await
    }

    // ── System ────────────────────────────────────────────────────────────────

    /// Get Tautulli server info and status.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_server_info(&self) -> Result<Value, TautulliError> {
        self.api_get("get_server_info", &[]).await
    }

    /// Get Tautulli settings.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn get_settings(&self) -> Result<Value, TautulliError> {
        self.api_get("get_settings", &[]).await
    }

    /// Health probe — calls `get_server_info` as the lightest authenticated check.
    ///
    /// # Errors
    /// Returns [`TautulliError::Api`] on HTTP failure.
    pub async fn probe(&self) -> Result<(), TautulliError> {
        drop(self.api_get("get_server_info", &[]).await?);
        Ok(())
    }
}
