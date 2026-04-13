//! Overseerr API client — media request management.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::OverseerrError;
use super::types::{
    CreateIssueBody, CreateRequestBody, Issue, IssueComment, IssueCommentBody, IssueList,
    MediaRequest, MovieDetail, OverseerrStatus, RequestList, SearchResponse, TvDetail, User,
    UserList,
};

/// Async client for the Overseerr REST API.
pub struct OverseerrClient {
    http: HttpClient,
}

impl OverseerrClient {
    /// Construct a new client targeting `base_url` with the given auth.
    ///
    /// Auth should be `Auth::ApiKey { header: "X-Api-Key".into(), key }`.
    ///
    /// # Errors
    /// Returns [`OverseerrError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Result<Self, OverseerrError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Status / health ────────────────────────────────────────────────────

    /// Fetch the current Overseerr status.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn status(&self) -> Result<OverseerrStatus, OverseerrError> {
        Ok(self.http.get_json("/api/v1/status").await?)
    }

    // ── Requests ──────────────────────────────────────────────────────────

    /// List media requests with optional filters.
    ///
    /// `take` and `skip` control pagination. `filter` can be one of:
    /// `all`, `approved`, `available`, `pending`, `processing`, `unavailable`, `failed`.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn request_list(
        &self,
        take: Option<u32>,
        skip: Option<u32>,
        filter: Option<&str>,
        sort: Option<&str>,
        requested_by: Option<u64>,
    ) -> Result<RequestList, OverseerrError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(t) = take {
            query.push(("take".into(), t.to_string()));
        }
        if let Some(s) = skip {
            query.push(("skip".into(), s.to_string()));
        }
        if let Some(f) = filter {
            query.push(("filter".into(), f.to_string()));
        }
        if let Some(s) = sort {
            query.push(("sort".into(), s.to_string()));
        }
        if let Some(u) = requested_by {
            query.push(("requestedBy".into(), u.to_string()));
        }
        Ok(self
            .http
            .get_json_query("/api/v1/request", &query)
            .await?)
    }

    /// Get a single request by ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport, not-found, or decode failure.
    pub async fn request_get(&self, id: u64) -> Result<MediaRequest, OverseerrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/request/{id}"))
            .await?)
    }

    /// Create a new media request.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport, validation, or decode failure.
    pub async fn request_create(&self, body: &CreateRequestBody) -> Result<MediaRequest, OverseerrError> {
        Ok(self.http.post_json("/api/v1/request", body).await?)
    }

    /// Approve a request by ID (sets status to `approved`).
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport failure.
    pub async fn request_approve(&self, id: u64) -> Result<MediaRequest, OverseerrError> {
        let body = serde_json::json!({});
        Ok(self
            .http
            .post_json(&format!("/api/v1/request/{id}/approve"), &body)
            .await?)
    }

    /// Decline a request by ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport failure.
    pub async fn request_decline(&self, id: u64) -> Result<MediaRequest, OverseerrError> {
        let body = serde_json::json!({});
        Ok(self
            .http
            .post_json(&format!("/api/v1/request/{id}/decline"), &body)
            .await?)
    }

    /// Delete a request by ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport failure.
    pub async fn request_delete(&self, id: u64) -> Result<(), OverseerrError> {
        Ok(self
            .http
            .delete(&format!("/api/v1/request/{id}"))
            .await?)
    }

    // ── Search ────────────────────────────────────────────────────────────

    /// Search for movies and TV shows.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn search(
        &self,
        query: &str,
        page: Option<u32>,
    ) -> Result<SearchResponse, OverseerrError> {
        let mut q: Vec<(String, String)> = vec![("query".into(), query.to_string())];
        if let Some(p) = page {
            q.push(("page".into(), p.to_string()));
        }
        Ok(self.http.get_json_query("/api/v1/search", &q).await?)
    }

    // ── Movies ────────────────────────────────────────────────────────────

    /// Get details for a movie by TMDB ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn movie_get(&self, tmdb_id: u64) -> Result<MovieDetail, OverseerrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/movie/{tmdb_id}"))
            .await?)
    }

    // ── TV ────────────────────────────────────────────────────────────────

    /// Get details for a TV show by TMDB ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn tv_get(&self, tmdb_id: u64) -> Result<TvDetail, OverseerrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/tv/{tmdb_id}"))
            .await?)
    }

    // ── Users ─────────────────────────────────────────────────────────────

    /// List all Overseerr users with pagination.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn user_list(
        &self,
        take: Option<u32>,
        skip: Option<u32>,
    ) -> Result<UserList, OverseerrError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(t) = take {
            query.push(("take".into(), t.to_string()));
        }
        if let Some(s) = skip {
            query.push(("skip".into(), s.to_string()));
        }
        Ok(self.http.get_json_query("/api/v1/user", &query).await?)
    }

    /// Get a single user by ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn user_get(&self, id: u64) -> Result<User, OverseerrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/user/{id}"))
            .await?)
    }

    // ── Issues ────────────────────────────────────────────────────────────

    /// List issues with optional filters.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn issue_list(
        &self,
        take: Option<u32>,
        skip: Option<u32>,
        filter: Option<&str>,
        sort: Option<&str>,
    ) -> Result<IssueList, OverseerrError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(t) = take {
            query.push(("take".into(), t.to_string()));
        }
        if let Some(s) = skip {
            query.push(("skip".into(), s.to_string()));
        }
        if let Some(f) = filter {
            query.push(("filter".into(), f.to_string()));
        }
        if let Some(s) = sort {
            query.push(("sort".into(), s.to_string()));
        }
        Ok(self.http.get_json_query("/api/v1/issue", &query).await?)
    }

    /// Get a single issue by ID.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn issue_get(&self, id: u64) -> Result<Issue, OverseerrError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/issue/{id}"))
            .await?)
    }

    /// Create a new issue.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn issue_create(&self, body: &CreateIssueBody) -> Result<Issue, OverseerrError> {
        Ok(self.http.post_json("/api/v1/issue", body).await?)
    }

    /// Add a comment to an issue.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport or decode failure.
    pub async fn issue_comment(
        &self,
        id: u64,
        message: &str,
    ) -> Result<IssueComment, OverseerrError> {
        let body = IssueCommentBody {
            message: message.to_string(),
        };
        Ok(self
            .http
            .post_json(&format!("/api/v1/issue/{id}/comment"), &body)
            .await?)
    }

    // ── Health probe ─────────────────────────────────────────────────────

    /// Probe health by fetching `/api/v1/status`.
    ///
    /// # Errors
    /// Returns [`OverseerrError`] on transport failure.
    pub async fn probe(&self) -> Result<(), OverseerrError> {
        drop(self.http.get_json::<Value>("/api/v1/status").await?);
        Ok(())
    }
}
