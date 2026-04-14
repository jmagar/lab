//! Overseerr request/response types.
//!
//! Derived from the upstream OpenAPI spec at <https://api.overseerr.dev/>.

use serde::{Deserialize, Serialize};

// ── Status ────────────────────────────────────────────────────────────────────

/// Overseerr application status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseerrStatus {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_tag: Option<String>,
    pub update_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_behind: Option<u64>,
    pub restart_required: bool,
}

// ── User ─────────────────────────────────────────────────────────────────────

/// Overseerr user record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plex_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_count: Option<u64>,
}

/// Paginated list of users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub page_info: Option<PageInfo>,
    pub results: Vec<User>,
}

// ── Page info ─────────────────────────────────────────────────────────────────

/// Pagination information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub pages: u64,
    pub page_size: u64,
    pub results: u64,
    pub page: u64,
}

// ── Request ───────────────────────────────────────────────────────────────────

/// A media request (movie or TV).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRequest {
    pub id: u64,
    pub status: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<User>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is4k: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<i32>>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// Paginated list of requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestList {
    pub page_info: Option<PageInfo>,
    pub results: Vec<MediaRequest>,
}

/// Body for creating a new request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequestBody {
    pub media_type: String,
    pub media_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasons: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is4k: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<i32>>,
}

// ── Search ─────────────────────────────────────────────────────────────────────

/// Search result (movie or TV show).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_air_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_info: Option<serde_json::Value>,
}

/// Paginated search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub page: u64,
    pub total_pages: u64,
    pub total_results: u64,
    pub results: Vec<SearchResult>,
}

// ── Movie detail ───────────────────────────────────────────────────────────────

/// Full movie details from Overseerr/TMDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieDetail {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_info: Option<serde_json::Value>,
}

// ── TV detail ─────────────────────────────────────────────────────────────────

/// Full TV show details from Overseerr/TMDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TvDetail {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_air_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_seasons: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_info: Option<serde_json::Value>,
}

// ── Issue ─────────────────────────────────────────────────────────────────────

/// An Overseerr issue report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: u64,
    pub issue_type: u32,
    pub status: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_season: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_episode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<IssueComment>>,
    pub created_at: String,
    pub updated_at: String,
}

/// An issue comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueComment {
    pub id: u64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated list of issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueList {
    pub page_info: Option<PageInfo>,
    pub results: Vec<Issue>,
}

/// Body for creating an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueBody {
    pub issue_type: u32,
    pub message: String,
    pub media_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_season: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_episode: Option<u32>,
}

/// Body for adding an issue comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCommentBody {
    pub message: String,
}

// ── Request counts ─────────────────────────────────────────────────────────────

/// Request counts by status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestCount {
    pub pending: u32,
    pub approved: u32,
    pub declined: u32,
    pub processing: u32,
    pub available: u32,
    pub total: u32,
}
