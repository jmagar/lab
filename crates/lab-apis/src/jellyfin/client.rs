//! Jellyfin read-first API methods.

use crate::core::{Auth, HttpClient};

use super::error::JellyfinError;
use super::types::{
    Item, ItemCounts, ItemsQuery, ItemsResult, PluginInfo, PublicSystemInfo, SessionInfo,
    SystemInfo, User, VirtualFolder,
};

/// Client for Jellyfin's HTTP API.
#[derive(Clone)]
pub struct JellyfinClient {
    http: HttpClient,
}

impl JellyfinClient {
    /// Build a client against a Jellyfin base URL.
    ///
    /// Use [`Self::auth_from_api_key`] to construct the first-wave API-key auth
    /// value for `JELLYFIN_API_KEY`.
    ///
    /// # Errors
    /// Returns [`JellyfinError::Api`] if the shared HTTP client cannot be built.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, JellyfinError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Build Jellyfin API-key auth for the `Authorization` header.
    ///
    /// The official OpenAPI scheme is `Authorization`; common Jellyfin
    /// deployments accept the `MediaBrowser Token="..."` value form.
    #[must_use]
    pub fn auth_from_api_key(api_key: &str) -> Auth {
        let value = api_key.trim();
        let key = if value.starts_with("MediaBrowser ") {
            value.to_string()
        } else {
            format!("MediaBrowser Token=\"{value}\"")
        };
        Auth::ApiKey {
            header: "Authorization".into(),
            key,
        }
    }

    /// Ping the Jellyfin server with the public health endpoint.
    pub async fn ping(&self) -> Result<String, JellyfinError> {
        self.http.get_text("/System/Ping").await.map_err(Into::into)
    }

    /// Fetch authenticated system information.
    pub async fn system_info(&self) -> Result<SystemInfo, JellyfinError> {
        self.http.get_json("/System/Info").await.map_err(Into::into)
    }

    /// Fetch public system information.
    pub async fn public_system_info(&self) -> Result<PublicSystemInfo, JellyfinError> {
        self.http
            .get_json("/System/Info/Public")
            .await
            .map_err(Into::into)
    }

    /// List Jellyfin users.
    pub async fn users(&self) -> Result<Vec<User>, JellyfinError> {
        self.http.get_json("/Users").await.map_err(Into::into)
    }

    /// Fetch the current authenticated user.
    pub async fn current_user(&self) -> Result<User, JellyfinError> {
        self.http.get_json("/Users/Me").await.map_err(Into::into)
    }

    /// List virtual folders/libraries.
    pub async fn libraries(&self) -> Result<Vec<VirtualFolder>, JellyfinError> {
        self.http
            .get_json("/Library/VirtualFolders")
            .await
            .map_err(Into::into)
    }

    /// Search or list items with a bounded first-wave query subset.
    pub async fn items(&self, query: &ItemsQuery) -> Result<ItemsResult, JellyfinError> {
        self.http
            .get_json_query("/Items", &query.to_pairs())
            .await
            .map_err(Into::into)
    }

    /// Fetch one item by Jellyfin item ID.
    pub async fn item(&self, item_id: &str) -> Result<Item, JellyfinError> {
        if item_id.trim().is_empty() {
            return Err(JellyfinError::InvalidParam(
                "item_id must not be empty".into(),
            ));
        }
        let encoded = HttpClient::encode_path_segment(item_id);
        self.http
            .get_json(&format!("/Items/{encoded}"))
            .await
            .map_err(Into::into)
    }

    /// Fetch item counts.
    pub async fn item_counts(&self) -> Result<ItemCounts, JellyfinError> {
        self.http
            .get_json("/Items/Counts")
            .await
            .map_err(Into::into)
    }

    /// List active sessions.
    pub async fn sessions(&self) -> Result<Vec<SessionInfo>, JellyfinError> {
        self.http.get_json("/Sessions").await.map_err(Into::into)
    }

    /// List installed plugins.
    pub async fn plugins(&self) -> Result<Vec<PluginInfo>, JellyfinError> {
        self.http.get_json("/Plugins").await.map_err(Into::into)
    }
}
