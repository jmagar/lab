//! Sonarr request/response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Series ────────────────────────────────────────────────────────────────────

/// A Sonarr TV series entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub monitored: bool,
    #[serde(rename = "seasonCount", default)]
    pub season_count: u32,
    #[serde(rename = "seriesType", default)]
    pub series_type: Option<String>,
    #[serde(rename = "qualityProfileId", default)]
    pub quality_profile_id: Option<i64>,
    #[serde(rename = "languageProfileId", default)]
    pub language_profile_id: Option<i64>,
    #[serde(rename = "rootFolderPath", default)]
    pub root_folder_path: Option<String>,
    #[serde(rename = "tvdbId", default)]
    pub tvdb_id: Option<i64>,
    #[serde(rename = "imdbId", default)]
    pub imdb_id: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(rename = "sizeOnDisk", default)]
    pub size_on_disk: Option<i64>,
}

/// Request body for adding a series to Sonarr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSeriesRequest {
    pub title: String,
    #[serde(rename = "tvdbId")]
    pub tvdb_id: i64,
    #[serde(rename = "qualityProfileId")]
    pub quality_profile_id: i64,
    #[serde(rename = "languageProfileId")]
    pub language_profile_id: i64,
    #[serde(rename = "rootFolderPath")]
    pub root_folder_path: String,
    pub monitored: bool,
    #[serde(rename = "seasonFolder")]
    pub season_folder: bool,
    #[serde(rename = "seriesType")]
    pub series_type: String,
    #[serde(rename = "addOptions")]
    pub add_options: Value,
}

// ── Episode ───────────────────────────────────────────────────────────────────

/// A Sonarr episode entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    #[serde(rename = "seriesId")]
    pub series_id: i64,
    #[serde(rename = "seasonNumber")]
    pub season_number: i32,
    #[serde(rename = "episodeNumber")]
    pub episode_number: i32,
    pub title: Option<String>,
    pub monitored: bool,
    #[serde(rename = "hasFile")]
    pub has_file: bool,
    #[serde(rename = "airDate", default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
}

// ── Queue ─────────────────────────────────────────────────────────────────────

/// Query params for the queue list endpoint.
#[derive(Debug, Clone, Default)]
pub struct QueueListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub series_id: Option<i64>,
}

impl QueueListQuery {
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(p) = self.page {
            pairs.push(("page".to_string(), p.to_string()));
        }
        if let Some(ps) = self.page_size {
            pairs.push(("pageSize".to_string(), ps.to_string()));
        }
        if let Some(sid) = self.series_id {
            pairs.push(("seriesId".to_string(), sid.to_string()));
        }
        pairs
    }
}

// ── History ───────────────────────────────────────────────────────────────────

/// Query params for the history list endpoint.
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub series_id: Option<i64>,
    pub episode_id: Option<i64>,
}

impl HistoryQuery {
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(p) = self.page {
            pairs.push(("page".to_string(), p.to_string()));
        }
        if let Some(ps) = self.page_size {
            pairs.push(("pageSize".to_string(), ps.to_string()));
        }
        if let Some(sid) = self.series_id {
            pairs.push(("seriesId".to_string(), sid.to_string()));
        }
        if let Some(eid) = self.episode_id {
            pairs.push(("episodeId".to_string(), eid.to_string()));
        }
        pairs
    }
}

// ── Calendar ──────────────────────────────────────────────────────────────────

/// Query params for the calendar list endpoint.
#[derive(Debug, Clone, Default)]
pub struct CalendarQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub unmonitored: Option<bool>,
}

impl CalendarQuery {
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(ref s) = self.start {
            pairs.push(("start".to_string(), s.clone()));
        }
        if let Some(ref e) = self.end {
            pairs.push(("end".to_string(), e.clone()));
        }
        if let Some(u) = self.unmonitored {
            pairs.push(("unmonitored".to_string(), u.to_string()));
        }
        pairs
    }
}
