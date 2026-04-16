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
        drop(self.http.get_text("/identity").await?);
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
        self.get_value(&format!("/library/sections/{section_id}"))
            .await
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
        self.get_value_query(&format!("/library/sections/{section_id}/refresh"), &query)
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
        self.get_value(&format!("/library/metadata/{rating_key}"))
            .await
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
        self.get_value_query("/status/sessions/terminate", &query)
            .await
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
        self.delete_no_body(&format!("/playlists/{playlist_id}"))
            .await
    }

    // ── Library browse / trash ────────────────────────────────────────────────

    /// List all content in a library section.
    ///
    /// Optional `type_filter` filters by media type (e.g. `"movie"`, `"show"`).
    /// Optional `sort` specifies the sort field (e.g. `"titleSort"`, `"addedAt:desc"`).
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_browse(
        &self,
        section_id: i64,
        type_filter: Option<&str>,
        sort: Option<&str>,
    ) -> Result<Value, PlexError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(t) = type_filter {
            query.push(("type".to_string(), t.to_string()));
        }
        if let Some(s) = sort {
            query.push(("sort".to_string(), s.to_string()));
        }
        self.get_value_query(&format!("/library/sections/{section_id}/all"), &query)
            .await
    }

    /// Empty the trash for a library section.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn library_empty_trash(&self, section_id: i64) -> Result<(), PlexError> {
        Ok(self
            .http
            .put_json::<Value, Value>(
                &format!("/library/sections/{section_id}/emptyTrash"),
                &Value::Null,
            )
            .await
            .map(|_| ())
            .or_else(|e| {
                // Plex returns 200 with empty body on success; tolerate decode errors
                use crate::core::ApiError;
                match e {
                    ApiError::Decode(_) => Ok(()),
                    other => Err(other),
                }
            })?)
    }

    // ── Metadata ──────────────────────────────────────────────────────────────

    /// Delete a metadata item by rating key.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn metadata_delete(&self, rating_key: &str) -> Result<(), PlexError> {
        self.delete_no_body(&format!("/library/metadata/{rating_key}"))
            .await
    }

    /// Edit (overwrite) metadata for an item using a raw JSON body.
    ///
    /// Plex accepts metadata edits as PUT query parameters; pass them as a
    /// flat JSON object and they will be forwarded as query params.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn metadata_edit(&self, rating_key: &str, body: &Value) -> Result<Value, PlexError> {
        let query: Vec<(String, String)> = body
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        let s = match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k.clone(), s)
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.get_value_query(&format!("/library/metadata/{rating_key}"), &query)
            .await
    }

    /// Trigger a metadata refresh for a specific item.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn metadata_refresh(&self, rating_key: &str) -> Result<(), PlexError> {
        Ok(self
            .http
            .put_json::<Value, Value>(
                &format!("/library/metadata/{rating_key}/refresh"),
                &Value::Null,
            )
            .await
            .map(|_| ())
            .or_else(|e| {
                use crate::core::ApiError;
                match e {
                    ApiError::Decode(_) => Ok(()),
                    other => Err(other),
                }
            })?)
    }

    // ── Session history ───────────────────────────────────────────────────────

    /// Get session history (recently played items).
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn session_history(
        &self,
        account_id: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Value, PlexError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(id) = account_id {
            query.push(("accountID".to_string(), id.to_string()));
        }
        if let Some(l) = limit {
            query.push(("X-Plex-Container-Size".to_string(), l.to_string()));
        }
        self.get_value_query("/status/sessions/history/all", &query)
            .await
    }

    // ── Hubs ──────────────────────────────────────────────────────────────────

    /// Get the Continue Watching hub items.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn hubs_continue_watching(&self) -> Result<Value, PlexError> {
        self.get_value("/hubs/continueWatching").await
    }

    // ── Butler ────────────────────────────────────────────────────────────────

    /// List all butler tasks and their status.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn butler_list(&self) -> Result<Value, PlexError> {
        self.get_value("/butler").await
    }

    /// Trigger a specific butler task by name.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn butler_run(&self, task_name: &str) -> Result<(), PlexError> {
        Ok(self
            .http
            .post_void(&format!("/butler/{task_name}"), &Value::Null)
            .await?)
    }

    // ── Scrobble / Unscrobble ─────────────────────────────────────────────────

    /// Mark an item as played (scrobble).
    ///
    /// Plex uses a GET to `/:/scrobble` with query params (not a POST).
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn item_scrobble(&self, rating_key: &str) -> Result<(), PlexError> {
        let query = vec![
            ("key".to_string(), rating_key.to_string()),
            (
                "identifier".to_string(),
                "com.plexapp.plugins.library".to_string(),
            ),
        ];
        // Plex returns 200 with empty or minimal body; use get_value_query and discard.
        drop(self.get_value_query("/:/scrobble", &query).await.ok());
        Ok(())
    }

    /// Mark an item as unplayed (unscrobble).
    ///
    /// Plex uses a GET to `/:/unscrobble` with query params (not a POST).
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn item_unscrobble(&self, rating_key: &str) -> Result<(), PlexError> {
        let query = vec![
            ("key".to_string(), rating_key.to_string()),
            (
                "identifier".to_string(),
                "com.plexapp.plugins.library".to_string(),
            ),
        ];
        drop(self.get_value_query("/:/unscrobble", &query).await.ok());
        Ok(())
    }

    // ── Updater ───────────────────────────────────────────────────────────────

    /// Get the current Plex Media Server update status.
    ///
    /// # Errors
    /// Returns `PlexError::Api` on HTTP failure.
    pub async fn updater_status(&self) -> Result<Value, PlexError> {
        self.get_value("/updater/status").await
    }
}
