//! `GotifyClient` — async methods against a Gotify server.
//!
//! Auth: pass `Auth::ApiKey { header: "X-Gotify-Key".into(), key: token }`.
//! An **app token** is required for `message_send`; a **client token** is
//! required for all other endpoints.

use crate::core::{Auth, HttpClient};

use super::error::GotifyError;
use super::types::{
    Application, ApplicationId, ApplicationParams, Client, ClientId, ClientParams, Health, Message,
    MessageId, PagedMessages, Plugin, PluginId, SendMessage, ServerVersion, UserCreate, UserId,
};

/// Client for a Gotify push-notification server.
pub struct GotifyClient {
    http: HttpClient,
}

impl GotifyClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// For most operations pass a **client token**:
    /// `Auth::ApiKey { header: "X-Gotify-Key".into(), key: client_token }`.
    ///
    /// For `message_send` the Gotify server expects an **app token**. If you
    /// need both operations from the same binary, construct two clients.
    ///
    /// # Errors
    /// Returns [`GotifyError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, GotifyError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Health ──────────────────────────────────────────────────────────────

    /// Fetch server health. Does not require authentication.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on transport failure.
    pub async fn server_health(&self) -> Result<Health, GotifyError> {
        Ok(self.http.get_json("/health").await?)
    }

    // ── Messages ────────────────────────────────────────────────────────────

    /// List all messages, newest first.
    ///
    /// `limit` caps the result count (server default: 100, max: 200).
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn messages_list(&self, limit: Option<u32>) -> Result<PagedMessages, GotifyError> {
        let path = limit.map_or_else(|| "/message".to_string(), |l| format!("/message?limit={l}"));
        Ok(self.http.get_json(&path).await?)
    }

    /// Send a push notification. Requires an **app token**.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn message_send(&self, msg: &SendMessage) -> Result<Message, GotifyError> {
        Ok(self.http.post_json("/message", msg).await?)
    }

    /// Delete a single message by id.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn message_delete(&self, id: MessageId) -> Result<(), GotifyError> {
        Ok(self.http.delete(&format!("/message/{}", id.0)).await?)
    }

    /// Delete **all** messages on the server.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn messages_purge(&self) -> Result<(), GotifyError> {
        Ok(self.http.delete("/message").await?)
    }

    // ── Application messages ────────────────────────────────────────────────

    /// List all messages for a specific application.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn app_messages_list(
        &self,
        app_id: ApplicationId,
    ) -> Result<PagedMessages, GotifyError> {
        Ok(self
            .http
            .get_json(&format!("/application/{}/message", app_id.0))
            .await?)
    }

    /// Delete **all** messages for a specific application.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn app_messages_delete(&self, app_id: ApplicationId) -> Result<(), GotifyError> {
        Ok(self
            .http
            .delete(&format!("/application/{}/message", app_id.0))
            .await?)
    }

    // ── Applications ────────────────────────────────────────────────────────

    /// List all applications.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn apps_list(&self) -> Result<Vec<Application>, GotifyError> {
        Ok(self.http.get_json("/application").await?)
    }

    /// Create an application and return it with its generated token.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn app_create(&self, params: &ApplicationParams) -> Result<Application, GotifyError> {
        Ok(self.http.post_json("/application", params).await?)
    }

    /// Update an application's name or description.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn app_update(
        &self,
        id: ApplicationId,
        params: &ApplicationParams,
    ) -> Result<Application, GotifyError> {
        Ok(self
            .http
            .put_json(&format!("/application/{}", id.0), params)
            .await?)
    }

    /// Delete an application and all its messages.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn app_delete(&self, id: ApplicationId) -> Result<(), GotifyError> {
        Ok(self.http.delete(&format!("/application/{}", id.0)).await?)
    }

    // ── Clients ─────────────────────────────────────────────────────────────

    /// List all clients.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn clients_list(&self) -> Result<Vec<Client>, GotifyError> {
        Ok(self.http.get_json("/client").await?)
    }

    /// Create a client and return it with its generated token.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn client_create(&self, params: &ClientParams) -> Result<Client, GotifyError> {
        Ok(self.http.post_json("/client", params).await?)
    }

    /// Update a client's name.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn client_update(
        &self,
        id: ClientId,
        params: &ClientParams,
    ) -> Result<Client, GotifyError> {
        Ok(self
            .http
            .put_json(&format!("/client/{}", id.0), params)
            .await?)
    }

    /// Delete a client.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn client_delete(&self, id: ClientId) -> Result<(), GotifyError> {
        Ok(self.http.delete(&format!("/client/{}", id.0)).await?)
    }

    // ── Plugins ─────────────────────────────────────────────────────────────

    /// List all server plugins.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn plugins_list(&self) -> Result<Vec<Plugin>, GotifyError> {
        Ok(self.http.get_json("/plugin").await?)
    }

    /// Enable a plugin.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn plugin_enable(&self, id: PluginId) -> Result<(), GotifyError> {
        Ok(self
            .http
            .post_void(&format!("/plugin/{}/enable", id.0), &serde_json::Value::Null)
            .await?)
    }

    /// Disable a plugin.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn plugin_disable(&self, id: PluginId) -> Result<(), GotifyError> {
        Ok(self
            .http
            .post_void(&format!("/plugin/{}/disable", id.0), &serde_json::Value::Null)
            .await?)
    }

    /// Get the configuration of a plugin as a YAML string.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn plugin_config_get(&self, id: PluginId) -> Result<String, GotifyError> {
        Ok(self
            .http
            .get_text(&format!("/plugin/{}/config", id.0))
            .await?)
    }

    /// Set the configuration of a plugin. Body must be valid YAML.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn plugin_config_set(&self, id: PluginId, config: &str) -> Result<(), GotifyError> {
        Ok(self
            .http
            .post_text_void(&format!("/plugin/{}/config", id.0), config)
            .await?)
    }

    // ── Users ────────────────────────────────────────────────────────────────

    /// List all users (admin only).
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn users_list(&self) -> Result<serde_json::Value, GotifyError> {
        Ok(self.http.get_json("/user").await?)
    }

    /// Create a user (admin only).
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn user_create(&self, params: &UserCreate) -> Result<serde_json::Value, GotifyError> {
        Ok(self.http.post_json("/user", params).await?)
    }

    /// Delete a user (admin only).
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn user_delete(&self, id: UserId) -> Result<(), GotifyError> {
        Ok(self.http.delete(&format!("/user/{}", id.0)).await?)
    }

    // ── Server ───────────────────────────────────────────────────────────────

    /// Fetch server version information.
    ///
    /// # Errors
    /// Returns `GotifyError::Api` on HTTP failure.
    pub async fn server_version(&self) -> Result<ServerVersion, GotifyError> {
        Ok(self.http.get_json("/version").await?)
    }
}
