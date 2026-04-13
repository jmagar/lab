//! `MemosClient` — memo management methods for the Memos v1 REST API.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::MemosError;
use super::types::{CreateMemoRequest, ListMemosParams, UpdateMemoRequest};

/// Client for a Memos instance (v1 API).
pub struct MemosClient {
    http: HttpClient,
}

impl MemosClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Memos uses open-id token auth: pass
    /// `Auth::Bearer { token: access_token }`.
    ///
    /// # Errors
    /// Returns [`MemosError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, MemosError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    async fn get_value(&self, path: &str) -> Result<Value, MemosError> {
        Ok(self.http.get_json(path).await?)
    }

    async fn get_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Value, MemosError> {
        Ok(self.http.get_json_query(path, query).await?)
    }

    async fn post_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, MemosError> {
        Ok(self.http.post_json(path, body).await?)
    }

    async fn patch_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, MemosError> {
        Ok(self.http.patch_json(path, body).await?)
    }

    async fn delete_value(&self, path: &str) -> Result<(), MemosError> {
        self.http.delete(path).await?;
        Ok(())
    }

    // ── Health ────────────────────────────────────────────────────────────

    /// Health probe — GET /api/v1/workspace/profile.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<Value, MemosError> {
        self.get_value("/api/v1/workspace/profile").await
    }

    // ── Memos ─────────────────────────────────────────────────────────────

    /// List memos.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memos_list(&self, params: &ListMemosParams) -> Result<Value, MemosError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(ref creator) = params.creator {
            query.push(("creator".to_string(), creator.clone()));
        }
        if let Some(ref tag) = params.tag {
            query.push(("tag".to_string(), tag.clone()));
        }
        if let Some(limit) = params.page_size {
            query.push(("pageSize".to_string(), limit.to_string()));
        }
        if let Some(ref token) = params.page_token {
            query.push(("pageToken".to_string(), token.clone()));
        }
        if query.is_empty() {
            self.get_value("/api/v1/memos").await
        } else {
            self.get_value_query("/api/v1/memos", &query).await
        }
    }

    /// Get a single memo by resource name (e.g. `"memos/123"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_get(&self, name: &str) -> Result<Value, MemosError> {
        self.get_value(&format!("/api/v1/{name}")).await
    }

    /// Create a new memo.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_create(&self, req: &CreateMemoRequest) -> Result<Value, MemosError> {
        self.post_value("/api/v1/memos", req).await
    }

    /// Update a memo by resource name.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_update(
        &self,
        name: &str,
        req: &UpdateMemoRequest,
    ) -> Result<Value, MemosError> {
        self.patch_value(&format!("/api/v1/{name}"), req).await
    }

    /// Delete a memo by resource name.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_delete(&self, name: &str) -> Result<(), MemosError> {
        self.delete_value(&format!("/api/v1/{name}")).await
    }

    // ── Tags ──────────────────────────────────────────────────────────────

    /// List all tags.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn tags_list(&self) -> Result<Value, MemosError> {
        self.get_value("/api/v1/tags").await
    }

    // ── Workspace ─────────────────────────────────────────────────────────

    /// Get workspace profile (name, version, owner).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn workspace_profile(&self) -> Result<Value, MemosError> {
        self.get_value("/api/v1/workspace/profile").await
    }

    // ── Users ─────────────────────────────────────────────────────────────

    /// Get the authenticated user's profile (`users/me`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn user_me(&self) -> Result<Value, MemosError> {
        self.get_value("/api/v1/users/me").await
    }
}
