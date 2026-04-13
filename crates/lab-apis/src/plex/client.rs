//! `PlexClient` — Plex Media Server methods.

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::PlexError;
use super::types::SearchParams;

/// Client for a Plex Media Server instance.
pub struct PlexClient {
    http: HttpClient,
}

impl PlexClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// Plex uses API-key style token auth: pass
    /// `Auth::ApiKey { header: "X-Plex-Token".into(), key: token }`.
    ///
    /// # Errors
    /// Returns [`PlexError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, PlexError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(Self {
            http: HttpClient::with_default_headers(base_url, auth, headers)?,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    async fn get_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Value, PlexError> {
        Ok(self.http.get_json_query(path, query).await?)
    }

    async fn delete_no_body(&self, path: &str) -> Result<(), PlexError> {
        Ok(self.http.delete(path).await?)
    }

    async fn get_value(&self, path: &str) -> Result<Value, PlexError> {
        Ok(self.http.get_json(path).await?)
    }

    // ── Server ────────────────────────────────────────────────────────────────

    /// Get server identity and capabilities.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn server_info(&self) -> Result<Value, PlexError> {
        self.get_value("/").await
    }

    /// Get server capabilities.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn server_capabilities(&self) -> Result<Value, PlexError> {
        self.get_value("/media/providers").await
    }

    /// Health probe — lightest authenticated request.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn probe(&self) -> Result<(), PlexError> {
        drop(self.get_value("/").await?);
        Ok(())
    }

    // ── Library ───────────────────────────────────────────────────────────────

    /// List all library sections.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_list(&self) -> Result<Value, PlexError> {
        self.get_value("/library/sections").await
    }

    /// Get metadata for a specific library section.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_get(&self, section_id: &str) -> Result<Value, PlexError> {
        self.get_value(&format!("/library/sections/{section_id}")).await
    }

    /// Trigger a scan of a library section for new content.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_scan(&self, section_id: &str) -> Result<Value, PlexError> {
        self.get_value(&format!("/library/sections/{section_id}/refresh"))
            .await
    }

    /// Refresh (force metadata re-download) for a library section.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_refresh(&self, section_id: &str) -> Result<Value, PlexError> {
        let query = vec![("force".to_string(), "1".to_string())];
        self.get_value_query(
            &format!("/library/sections/{section_id}/refresh"),
            &query,
        )
        .await
    }

    // ── Media ─────────────────────────────────────────────────────────────────

    /// Search for media across all or a specific library section.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn media_search(&self, params: &SearchParams) -> Result<Value, PlexError> {
        let mut query = vec![("query".to_string(), params.query.clone())];
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref section_id) = params.section_id {
            query.push(("sectionId".to_string(), section_id.clone()));
        }
        self.get_value_query("/hubs/search", &query).await
    }

    /// Get metadata for a specific media item by rating key.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn media_get(&self, rating_key: &str) -> Result<Value, PlexError> {
        self.get_value(&format!("/library/metadata/{rating_key}")).await
    }

    // ── Sessions ──────────────────────────────────────────────────────────────

    /// List active playback sessions.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn session_list(&self) -> Result<Value, PlexError> {
        self.get_value("/status/sessions").await
    }

    /// Terminate an active playback session.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn session_terminate(
        &self,
        session_id: &str,
        reason: Option<&str>,
    ) -> Result<Value, PlexError> {
        let mut query = vec![("sessionId".to_string(), session_id.to_string())];
        if let Some(r) = reason {
            query.push(("reason".to_string(), r.to_string()));
        }
        self.get_value_query("/status/sessions/terminate", &query).await
    }

    // ── Playlists ─────────────────────────────────────────────────────────────

    /// List all playlists.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn playlist_list(&self) -> Result<Value, PlexError> {
        self.get_value("/playlists").await
    }

    /// Get a specific playlist by rating key.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn playlist_get(&self, playlist_id: &str) -> Result<Value, PlexError> {
        self.get_value(&format!("/playlists/{playlist_id}")).await
    }

    /// Create a new playlist.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn playlist_create(
        &self,
        title: &str,
        playlist_type: &str,
        uri: Option<&str>,
    ) -> Result<Value, PlexError> {
        let mut query = vec![
            ("title".to_string(), title.to_string()),
            ("type".to_string(), playlist_type.to_string()),
            ("smart".to_string(), "0".to_string()),
        ];
        if let Some(u) = uri {
            query.push(("uri".to_string(), u.to_string()));
        }
        self.get_value_query("/playlists", &query).await
    }

    /// Delete a playlist by rating key.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn playlist_delete(&self, playlist_id: &str) -> Result<(), PlexError> {
        self.delete_no_body(&format!("/playlists/{playlist_id}")).await
    }
}
