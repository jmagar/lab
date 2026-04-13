//! `LinkdingClient` — bookmark management methods.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::LinkdingError;
use super::types::{
    BookmarkListParams, BookmarkUpdateRequest, BookmarkWriteRequest, TagCreateRequest,
};

/// Client for a Linkding bookmark manager instance.
pub struct LinkdingClient {
    http: HttpClient,
}

impl LinkdingClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Linkding uses token auth: `Authorization: Token <api_token>`.
    /// Pass `Auth::ApiKey { header: "Authorization".into(), key: format!("Token {token}") }`.
    ///
    /// # Errors
    /// Returns [`LinkdingError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, LinkdingError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    async fn get_value(&self, path: &str) -> Result<Value, LinkdingError> {
        Ok(self.http.get_json(path).await?)
    }

    async fn get_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Value, LinkdingError> {
        Ok(self.http.get_json_query(path, query).await?)
    }

    async fn post_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, LinkdingError> {
        Ok(self.http.post_json(path, body).await?)
    }

    async fn patch_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, LinkdingError> {
        Ok(self.http.patch_json(path, body).await?)
    }

    async fn delete_value(&self, path: &str) -> Result<(), LinkdingError> {
        Ok(self.http.delete(path).await?)
    }

    /// Build a query string from bookmark list params.
    fn bookmark_list_query(params: &BookmarkListParams) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref search) = params.q {
            q.push(("q".to_string(), search.clone()));
        }
        if let Some(limit) = params.limit {
            q.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = params.offset {
            q.push(("offset".to_string(), offset.to_string()));
        }
        q
    }

    /// Health probe — lightest authenticated request.
    ///
    /// Uses `GET /api/bookmarks/?limit=1` as the probe target.
    ///
    /// # Errors
    /// Returns `LinkdingError::Api` on HTTP failure.
    pub async fn probe(&self) -> Result<(), LinkdingError> {
        let query = vec![("limit".to_string(), "1".to_string())];
        drop(self.get_value_query("/api/bookmarks/", &query).await?);
        Ok(())
    }

    // ── Bookmarks ──────────────────────────────────────────────────────────

    /// List bookmarks with optional filters.
    pub async fn bookmarks_list(
        &self,
        params: &BookmarkListParams,
    ) -> Result<Value, LinkdingError> {
        let query = Self::bookmark_list_query(params);
        self.get_value_query("/api/bookmarks/", &query).await
    }

    /// List archived bookmarks with optional filters.
    pub async fn bookmarks_archived_list(
        &self,
        params: &BookmarkListParams,
    ) -> Result<Value, LinkdingError> {
        let query = Self::bookmark_list_query(params);
        self.get_value_query("/api/bookmarks/archived/", &query)
            .await
    }

    /// Retrieve a single bookmark by ID.
    pub async fn bookmark_get(&self, id: u64) -> Result<Value, LinkdingError> {
        self.get_value(&format!("/api/bookmarks/{id}/")).await
    }

    /// Check whether a URL is already bookmarked.
    ///
    /// Returns bookmark data (if found), scraped metadata, and auto-tags.
    pub async fn bookmark_check(&self, url: &str) -> Result<Value, LinkdingError> {
        let query = vec![("url".to_string(), url.to_string())];
        self.get_value_query("/api/bookmarks/check/", &query).await
    }

    /// Create a new bookmark.
    pub async fn bookmark_create(
        &self,
        body: &BookmarkWriteRequest,
    ) -> Result<Value, LinkdingError> {
        self.post_value("/api/bookmarks/", body).await
    }

    /// Partially update a bookmark (PATCH — only provided fields are changed).
    pub async fn bookmark_update(
        &self,
        id: u64,
        body: &BookmarkUpdateRequest,
    ) -> Result<Value, LinkdingError> {
        self.patch_value(&format!("/api/bookmarks/{id}/"), body)
            .await
    }

    /// Archive a bookmark.
    pub async fn bookmark_archive(&self, id: u64) -> Result<Value, LinkdingError> {
        self.post_value(
            &format!("/api/bookmarks/{id}/archive/"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Unarchive a bookmark.
    pub async fn bookmark_unarchive(&self, id: u64) -> Result<Value, LinkdingError> {
        self.post_value(
            &format!("/api/bookmarks/{id}/unarchive/"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Delete a bookmark by ID.
    pub async fn bookmark_delete(&self, id: u64) -> Result<(), LinkdingError> {
        self.delete_value(&format!("/api/bookmarks/{id}/")).await
    }

    // ── Tags ──────────────────────────────────────────────────────────────

    /// List all tags.
    pub async fn tags_list(&self) -> Result<Value, LinkdingError> {
        self.get_value("/api/tags/").await
    }

    /// Retrieve a single tag by ID.
    pub async fn tag_get(&self, id: u64) -> Result<Value, LinkdingError> {
        self.get_value(&format!("/api/tags/{id}/")).await
    }

    /// Create a new tag.
    pub async fn tag_create(&self, body: &TagCreateRequest) -> Result<Value, LinkdingError> {
        self.post_value("/api/tags/", body).await
    }

    // ── User ──────────────────────────────────────────────────────────────

    /// Retrieve user profile / preferences.
    pub async fn user_profile(&self) -> Result<Value, LinkdingError> {
        self.get_value("/api/user/profile/").await
    }
}
