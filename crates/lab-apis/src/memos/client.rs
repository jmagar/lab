//! `MemosClient` — memo management methods for the Memos v1 REST API.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::MemosError;
use super::types::{
    Attachment, Comment, CreateCommentRequest, CreateMemoRequest, CreateWebhookRequest,
    ListMemosParams, MemoUser, ShareLink, UpdateMemoRequest, UserStats, Webhook,
};

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

    /// Get the authenticated user's profile (`auth/me`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn user_me(&self) -> Result<Value, MemosError> {
        self.get_value("/api/v1/auth/me").await
    }

    /// List all users (admin only; propagates 403 as `MemosError::Api(ApiError::Auth)`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure, including 403 for non-admins.
    pub async fn users_list(&self) -> Result<Vec<MemoUser>, MemosError> {
        Ok(self.http.get_json("/api/v1/users").await?)
    }

    /// Get statistics for a user by resource name (e.g. `"users/1"` or `"users/me"`).
    ///
    /// Uses the Google-API-style colon path: `/api/v1/{user}:getStats`.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn user_stats(&self, user: &str) -> Result<UserStats, MemosError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/{user}:getStats"))
            .await?)
    }

    // ── Webhooks ──────────────────────────────────────────────────────────

    /// List webhooks for a user by resource name (e.g. `"users/1"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn webhooks_list(&self, user: &str) -> Result<Vec<Webhook>, MemosError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/{user}/webhooks"))
            .await?)
    }

    /// Create a webhook for a user.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn webhook_create(
        &self,
        user: &str,
        req: &CreateWebhookRequest,
    ) -> Result<Webhook, MemosError> {
        Ok(self
            .http
            .post_json(&format!("/api/v1/{user}/webhooks"), req)
            .await?)
    }

    // ── Attachments ───────────────────────────────────────────────────────

    /// Upload a file attachment via multipart/form-data POST.
    ///
    /// The `file_bytes` are the raw file content, `filename` is the original name,
    /// and `mime_type` is the MIME type (e.g. `"image/png"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn attachment_upload(
        &self,
        filename: &str,
        file_bytes: Vec<u8>,
        mime_type: &str,
    ) -> Result<Attachment, MemosError> {
        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(filename.to_string())
            .mime_str(mime_type)
            .map_err(|e| crate::core::ApiError::Internal(format!("invalid mime type: {e}")))?;
        let form = reqwest::multipart::Form::new().part("file", part);
        Ok(self
            .http
            .post_multipart("/api/v1/attachments", form)
            .await?)
    }

    /// Delete an attachment by resource name (e.g. `"attachments/123"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn attachment_delete(&self, name: &str) -> Result<(), MemosError> {
        self.delete_value(&format!("/api/v1/{name}")).await
    }

    // ── Memo comments ─────────────────────────────────────────────────────

    /// List comments for a memo by memo resource name (e.g. `"memos/123"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_comments_list(&self, memo_name: &str) -> Result<Vec<Comment>, MemosError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/{memo_name}/comments"))
            .await?)
    }

    /// Create a comment on a memo.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_comment_create(
        &self,
        memo_name: &str,
        req: &CreateCommentRequest,
    ) -> Result<Comment, MemosError> {
        Ok(self
            .http
            .post_json(&format!("/api/v1/{memo_name}/comments"), req)
            .await?)
    }

    // ── Memo shares ───────────────────────────────────────────────────────

    /// List share links for a memo by memo resource name (e.g. `"memos/123"`).
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_shares_list(&self, memo_name: &str) -> Result<Vec<ShareLink>, MemosError> {
        Ok(self
            .http
            .get_json(&format!("/api/v1/{memo_name}/shares"))
            .await?)
    }

    /// Create a share link for a memo.
    ///
    /// # Errors
    /// Returns `MemosError::Api` on HTTP failure.
    pub async fn memo_share_create(&self, memo_name: &str) -> Result<ShareLink, MemosError> {
        // POST with an empty body — Memos creates a share link without parameters.
        Ok(self
            .http
            .post_json(
                &format!("/api/v1/{memo_name}/shares"),
                &serde_json::json!({}),
            )
            .await?)
    }
}
