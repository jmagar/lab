//! `PaperlessClient` — document management methods.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::PaperlessError;
use super::types::{
    CorrespondentCreateRequest, DocumentTypeCreateRequest, DocumentUpdateRequest, TagCreateRequest,
};

/// Client for a Paperless-ngx instance.
pub struct PaperlessClient {
    http: HttpClient,
}

impl PaperlessClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Paperless-ngx uses API token auth. Pass:
    /// ```text
    /// Auth::ApiKey { header: "Authorization".into(), key: format!("Token {token}") }
    /// ```
    ///
    /// # Errors
    /// Returns [`PaperlessError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, PaperlessError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    async fn get_value(&self, path: &str) -> Result<Value, PaperlessError> {
        Ok(self.http.get_json(path).await?)
    }

    async fn post_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, PaperlessError> {
        Ok(self.http.post_json(path, body).await?)
    }

    async fn patch_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, PaperlessError> {
        Ok(self.http.patch_json(path, body).await?)
    }

    async fn delete_value(&self, path: &str) -> Result<(), PaperlessError> {
        Ok(self.http.delete(path).await?)
    }

    // ── Health ────────────────────────────────────────────────────────────────

    /// Health probe via `GET /api/statistics/` — lightest authenticated endpoint.
    ///
    /// # Errors
    /// Returns `PaperlessError::Api` on HTTP failure.
    pub async fn probe(&self) -> Result<(), PaperlessError> {
        drop(self.get_value("/api/statistics/").await?);
        Ok(())
    }

    // ── Documents ─────────────────────────────────────────────────────────────

    /// List documents.
    ///
    /// Optional query parameters are passed as `params` — a flat JSON object
    /// whose keys map to query-string fields (`query`, `page`, `page_size`, etc.).
    pub async fn documents_list(&self, params: &Value) -> Result<Value, PaperlessError> {
        let qs = build_query_string(params, &[
            "query",
            "page",
            "page_size",
            "ordering",
            "correspondent",
            "document_type",
            "tags__id__all",
        ]);
        self.get_value(&format!("/api/documents/{qs}")).await
    }

    /// Get a single document by ID.
    pub async fn document_get(&self, id: u64) -> Result<Value, PaperlessError> {
        self.get_value(&format!("/api/documents/{id}/")).await
    }

    /// Get full metadata for a document.
    pub async fn document_metadata(&self, id: u64) -> Result<Value, PaperlessError> {
        self.get_value(&format!("/api/documents/{id}/metadata/"))
            .await
    }

    /// Partially update a document (PATCH).
    pub async fn document_update(
        &self,
        id: u64,
        body: &DocumentUpdateRequest,
    ) -> Result<Value, PaperlessError> {
        self.patch_value(&format!("/api/documents/{id}/"), body)
            .await
    }

    /// Delete a document.
    pub async fn document_delete(&self, id: u64) -> Result<(), PaperlessError> {
        self.delete_value(&format!("/api/documents/{id}/")).await
    }

    // ── Tags ──────────────────────────────────────────────────────────────────

    /// List all tags.
    pub async fn tags_list(&self) -> Result<Value, PaperlessError> {
        self.get_value("/api/tags/").await
    }

    /// Get one tag by ID.
    pub async fn tag_get(&self, id: u64) -> Result<Value, PaperlessError> {
        self.get_value(&format!("/api/tags/{id}/")).await
    }

    /// Create a tag.
    pub async fn tag_create(&self, body: &TagCreateRequest) -> Result<Value, PaperlessError> {
        self.post_value("/api/tags/", body).await
    }

    /// Delete a tag.
    pub async fn tag_delete(&self, id: u64) -> Result<(), PaperlessError> {
        self.delete_value(&format!("/api/tags/{id}/")).await
    }

    // ── Correspondents ────────────────────────────────────────────────────────

    /// List all correspondents.
    pub async fn correspondents_list(&self) -> Result<Value, PaperlessError> {
        self.get_value("/api/correspondents/").await
    }

    /// Get one correspondent by ID.
    pub async fn correspondent_get(&self, id: u64) -> Result<Value, PaperlessError> {
        self.get_value(&format!("/api/correspondents/{id}/")).await
    }

    /// Create a correspondent.
    pub async fn correspondent_create(
        &self,
        body: &CorrespondentCreateRequest,
    ) -> Result<Value, PaperlessError> {
        self.post_value("/api/correspondents/", body).await
    }

    /// Delete a correspondent.
    pub async fn correspondent_delete(&self, id: u64) -> Result<(), PaperlessError> {
        self.delete_value(&format!("/api/correspondents/{id}/"))
            .await
    }

    // ── Document Types ────────────────────────────────────────────────────────

    /// List all document types.
    pub async fn document_types_list(&self) -> Result<Value, PaperlessError> {
        self.get_value("/api/document_types/").await
    }

    /// Get one document type by ID.
    pub async fn document_type_get(&self, id: u64) -> Result<Value, PaperlessError> {
        self.get_value(&format!("/api/document_types/{id}/")).await
    }

    /// Create a document type.
    pub async fn document_type_create(
        &self,
        body: &DocumentTypeCreateRequest,
    ) -> Result<Value, PaperlessError> {
        self.post_value("/api/document_types/", body).await
    }

    /// Delete a document type.
    pub async fn document_type_delete(&self, id: u64) -> Result<(), PaperlessError> {
        self.delete_value(&format!("/api/document_types/{id}/"))
            .await
    }

    // ── Statistics & Tasks ────────────────────────────────────────────────────

    /// Get instance statistics.
    pub async fn statistics(&self) -> Result<Value, PaperlessError> {
        self.get_value("/api/statistics/").await
    }

    /// List async tasks.
    pub async fn tasks_list(&self) -> Result<Value, PaperlessError> {
        self.get_value("/api/tasks/").await
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a URL query string from the allowed keys present in `params`.
///
/// Returns `""` when no matching keys are found, or `"?key=value&..."` otherwise.
fn build_query_string(params: &Value, allowed: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for &key in allowed {
        if let Some(v) = params.get(key) {
            let encoded = match v {
                Value::String(s) => urlencoding(s),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| match item {
                        Value::String(s) => Some(urlencoding(s)),
                        Value::Number(n) => Some(n.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(","),
                _ => continue,
            };
            parts.push(format!("{key}={encoded}"));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

/// Minimal percent-encoding for query string values.
fn urlencoding(s: &str) -> String {
    // Use a simple approach: encode the most critical characters.
    // For homelab use cases this is sufficient; a full percent-encoder
    // would require an additional dep (`percent-encoding`).
    s.replace('%', "%25")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace('/', "%2F")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('+', "%2B")
        .replace(' ', "+")
}
