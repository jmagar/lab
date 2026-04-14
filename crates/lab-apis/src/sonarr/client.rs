//! `SonarrClient` — TV series management methods.
//!
//! Implements the Sonarr v3 API for series, episodes, queue, history,
//! calendar, tags, root folders, quality profiles, language profiles,
//! system status, and health.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::SonarrError;
use super::types::{
    AddSeriesRequest, CalendarQuery, Episode, HistoryQuery, QueueListQuery, Series,
};

/// Client for a Sonarr TV series manager instance.
pub struct SonarrClient {
    pub(crate) http: HttpClient,
}

impl SonarrClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Sonarr uses header-token auth: pass
    /// `Auth::ApiKey { header: "X-Api-Key".into(), key: api_key }`.
    ///
    /// # Errors
    /// Returns [`SonarrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, SonarrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Series ────────────────────────────────────────────────────────────────

    /// List all series in the Sonarr library.
    ///
    /// `GET /api/v3/series`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_list(&self) -> Result<Vec<Series>, SonarrError> {
        Ok(self.http.get_json("/api/v3/series").await?)
    }

    /// Get a single series by ID.
    ///
    /// `GET /api/v3/series/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_get(&self, id: i64) -> Result<Series, SonarrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v3/series/{id}"))
            .await?)
    }

    /// Lookup series candidates by TVDB ID or search term.
    ///
    /// `GET /api/v3/series/lookup?term={query}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_lookup(&self, query: &str) -> Result<Value, SonarrError> {
        Ok(self
            .http
            .get_json_query("/api/v3/series/lookup", &[("term".to_string(), query.to_string())])
            .await?)
    }

    /// Add a series to Sonarr for monitoring and download.
    ///
    /// `POST /api/v3/series`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_add(&self, req: &AddSeriesRequest) -> Result<Series, SonarrError> {
        Ok(self.http.post_json("/api/v3/series", req).await?)
    }

    /// Delete a series from Sonarr.
    ///
    /// `DELETE /api/v3/series/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_delete(&self, id: i64, delete_files: bool) -> Result<(), SonarrError> {
        let pairs = vec![("deleteFiles".to_string(), delete_files.to_string())];
        Ok(self
            .http
            .delete_query(&format!("/api/v3/series/{id}"), &pairs)
            .await?)
    }

    // ── Episodes ──────────────────────────────────────────────────────────────

    /// List episodes for a given series.
    ///
    /// `GET /api/v3/episode?seriesId={series_id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn episode_list(&self, series_id: i64) -> Result<Vec<Episode>, SonarrError> {
        Ok(self
            .http
            .get_json_query(
                "/api/v3/episode",
                &[("seriesId".to_string(), series_id.to_string())],
            )
            .await?)
    }

    /// Get a single episode by ID.
    ///
    /// `GET /api/v3/episode/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn episode_get(&self, id: i64) -> Result<Episode, SonarrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v3/episode/{id}"))
            .await?)
    }

    // ── Queue ─────────────────────────────────────────────────────────────────

    /// List the download queue.
    ///
    /// `GET /api/v3/queue`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn queue_list(&self, query: &QueueListQuery) -> Result<Value, SonarrError> {
        let pairs = query.to_query_pairs();
        Ok(self.http.get_json_query("/api/v3/queue", &pairs).await?)
    }

    /// Remove an item from the download queue by ID.
    ///
    /// `DELETE /api/v3/queue/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn queue_delete(&self, id: i64, blocklist: bool) -> Result<(), SonarrError> {
        let pairs = vec![("blocklist".to_string(), blocklist.to_string())];
        Ok(self
            .http
            .delete_query(&format!("/api/v3/queue/{id}"), &pairs)
            .await?)
    }

    // ── History ───────────────────────────────────────────────────────────────

    /// List history with optional filters.
    ///
    /// `GET /api/v3/history`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn history_list(&self, query: &HistoryQuery) -> Result<Value, SonarrError> {
        let pairs = query.to_query_pairs();
        Ok(self.http.get_json_query("/api/v3/history", &pairs).await?)
    }

    // ── Wanted ────────────────────────────────────────────────────────────────

    /// List wanted/missing episodes.
    ///
    /// `GET /api/v3/wanted/missing`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn wanted_missing(&self, page: Option<u32>, page_size: Option<u32>) -> Result<Value, SonarrError> {
        let mut pairs = Vec::new();
        if let Some(p) = page {
            pairs.push(("page".to_string(), p.to_string()));
        }
        if let Some(ps) = page_size {
            pairs.push(("pageSize".to_string(), ps.to_string()));
        }
        Ok(self.http.get_json_query("/api/v3/wanted/missing", &pairs).await?)
    }

    // ── Calendar ──────────────────────────────────────────────────────────────

    /// List episodes in the calendar window.
    ///
    /// `GET /api/v3/calendar`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn calendar_list(&self, query: &CalendarQuery) -> Result<Value, SonarrError> {
        let pairs = query.to_query_pairs();
        Ok(self.http.get_json_query("/api/v3/calendar", &pairs).await?)
    }

    // ── Health ────────────────────────────────────────────────────────────────

    /// Get system health alerts.
    ///
    /// `GET /api/v3/health`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/health").await?)
    }

    // ── System Status ─────────────────────────────────────────────────────────

    /// Get system status (version, branch, OS, etc.).
    ///
    /// `GET /api/v3/system/status`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn system_status(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/system/status").await?)
    }

    /// Probe via system status (lightest authenticated endpoint).
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn probe(&self) -> Result<(), SonarrError> {
        self.http.get_void("/api/v3/system/status").await?;
        Ok(())
    }

    // ── Tags ──────────────────────────────────────────────────────────────────

    /// List all tags.
    ///
    /// `GET /api/v3/tag`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn tag_list(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/tag").await?)
    }

    /// Create a new tag.
    ///
    /// `POST /api/v3/tag`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn tag_create(&self, label: &str) -> Result<Value, SonarrError> {
        let body = serde_json::json!({ "id": 0, "label": label });
        Ok(self.http.post_json("/api/v3/tag", &body).await?)
    }

    /// Delete a tag by ID.
    ///
    /// `DELETE /api/v3/tag/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn tag_delete(&self, id: i64) -> Result<(), SonarrError> {
        Ok(self.http.delete(&format!("/api/v3/tag/{id}")).await?)
    }

    // ── Root Folders ──────────────────────────────────────────────────────────

    /// List all root folders.
    ///
    /// `GET /api/v3/rootfolder`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn rootfolder_list(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/rootfolder").await?)
    }

    // ── Quality Profiles ──────────────────────────────────────────────────────

    /// List all quality profiles.
    ///
    /// `GET /api/v3/qualityprofile`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn qualityprofile_list(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/qualityprofile").await?)
    }

    // ── Language Profiles ─────────────────────────────────────────────────────

    /// List all language profiles.
    ///
    /// `GET /api/v3/languageprofile`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn languageprofile_list(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/languageprofile").await?)
    }

    // ── Series Edit ───────────────────────────────────────────────────────────

    /// Update an existing series by ID with a full series resource body.
    ///
    /// `PUT /api/v3/series/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn series_edit(&self, id: i64, body: &Value) -> Result<Value, SonarrError> {
        Ok(self.http.put_json(&format!("/api/v3/series/{id}"), body).await?)
    }

    // ── Episode Monitor ───────────────────────────────────────────────────────

    /// Set the monitored state for a list of episodes.
    ///
    /// `PUT /api/v3/episode/monitor`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn episode_monitor(
        &self,
        episode_ids: &[i64],
        monitored: bool,
    ) -> Result<Value, SonarrError> {
        let body = serde_json::json!({ "episodeIds": episode_ids, "monitored": monitored });
        Ok(self.http.put_json("/api/v3/episode/monitor", &body).await?)
    }

    // ── Wanted Cutoff ─────────────────────────────────────────────────────────

    /// List episodes that have not met their cutoff quality.
    ///
    /// `GET /api/v3/wanted/cutoff`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn wanted_cutoff(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Value, SonarrError> {
        let mut pairs = Vec::new();
        if let Some(p) = page {
            pairs.push(("page".to_string(), p.to_string()));
        }
        if let Some(ps) = page_size {
            pairs.push(("pageSize".to_string(), ps.to_string()));
        }
        Ok(self.http.get_json_query("/api/v3/wanted/cutoff", &pairs).await?)
    }

    // ── Releases ──────────────────────────────────────────────────────────────

    /// Search for available releases for a series/season.
    ///
    /// `GET /api/v3/release?seriesId=&seasonNumber=`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn release_search(
        &self,
        series_id: Option<i64>,
        season_number: Option<i32>,
    ) -> Result<Value, SonarrError> {
        let mut pairs = Vec::new();
        if let Some(sid) = series_id {
            pairs.push(("seriesId".to_string(), sid.to_string()));
        }
        if let Some(sn) = season_number {
            pairs.push(("seasonNumber".to_string(), sn.to_string()));
        }
        Ok(self.http.get_json_query("/api/v3/release", &pairs).await?)
    }

    /// Grab a release by GUID.
    ///
    /// `POST /api/v3/release`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn release_grab(&self, body: &Value) -> Result<Value, SonarrError> {
        Ok(self.http.post_json("/api/v3/release", body).await?)
    }

    // ── History ───────────────────────────────────────────────────────────────

    /// List history records for a specific series.
    ///
    /// `GET /api/v3/history/series?seriesId={id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn history_series(&self, series_id: i64) -> Result<Value, SonarrError> {
        Ok(self
            .http
            .get_json_query(
                "/api/v3/history/series",
                &[("seriesId".to_string(), series_id.to_string())],
            )
            .await?)
    }

    /// Retry a failed history item by ID.
    ///
    /// `POST /api/v3/history/failed/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn history_failed_retry(&self, id: i64) -> Result<(), SonarrError> {
        let body = serde_json::json!({});
        Ok(self
            .http
            .post_void(&format!("/api/v3/history/failed/{id}"), &body)
            .await?)
    }

    // ── Blocklist ─────────────────────────────────────────────────────────────

    /// List all blocklisted releases.
    ///
    /// `GET /api/v3/blocklist`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn blocklist_list(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/blocklist").await?)
    }

    /// Remove a release from the blocklist by ID.
    ///
    /// `DELETE /api/v3/blocklist/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn blocklist_delete(&self, id: i64) -> Result<(), SonarrError> {
        Ok(self.http.delete(&format!("/api/v3/blocklist/{id}")).await?)
    }

    // ── Episode Files ─────────────────────────────────────────────────────────

    /// Delete an episode file by ID.
    ///
    /// `DELETE /api/v3/episodefile/{id}`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn episodefile_delete(&self, id: i64) -> Result<(), SonarrError> {
        Ok(self.http.delete(&format!("/api/v3/episodefile/{id}")).await?)
    }

    // ── System ────────────────────────────────────────────────────────────────

    /// Restart the Sonarr application.
    ///
    /// `POST /api/v3/system/restart`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn system_restart(&self) -> Result<(), SonarrError> {
        let body = serde_json::json!({});
        Ok(self.http.post_void("/api/v3/system/restart", &body).await?)
    }

    /// List system backup files.
    ///
    /// `GET /api/v3/system/backup`
    ///
    /// # Errors
    /// Returns `SonarrError::Api` on HTTP failure.
    pub async fn system_backup(&self) -> Result<Value, SonarrError> {
        Ok(self.http.get_json("/api/v3/system/backup").await?)
    }
}
