//! `OpenAcpClient` - upstream OpenACP daemon REST API.

use serde::Serialize;
use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::error::OpenAcpError;
use super::types::{
    AdoptSessionRequest, BypassRequest, ConfigPatchRequest, CreateSessionRequest, NotifyRequest,
    OpenAcpValue, PermissionResolveRequest, PromptRequest, TopicsCleanupRequest,
    TunnelCreateRequest,
};

/// Client for an upstream OpenACP daemon.
#[derive(Debug, Clone)]
pub struct OpenAcpClient {
    http: HttpClient,
}

impl OpenAcpClient {
    /// Build a client against `base_url` with the given auth strategy.
    ///
    /// Protected endpoints use `Auth::Bearer`; `system.health` and
    /// `system.version` also work with `Auth::None`.
    ///
    /// # Errors
    /// Returns [`OpenAcpError::Api`] if the HTTP client cannot be constructed.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, OpenAcpError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    async fn get_value(&self, path: &str) -> Result<OpenAcpValue, OpenAcpError> {
        Ok(self.http.get_json(path).await?)
    }

    async fn get_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<OpenAcpValue, OpenAcpError> {
        Ok(self.http.get_json_query(path, query).await?)
    }

    async fn post_value<B: Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        Ok(self.http.post_json(path, body).await?)
    }

    async fn patch_value<B: Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        Ok(self.http.patch_json(path, body).await?)
    }

    async fn delete_value(&self, path: &str) -> Result<(), OpenAcpError> {
        Ok(self.http.delete(path).await?)
    }

    /// Return daemon health. No auth is required by upstream OpenACP.
    pub async fn health(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/health").await
    }

    /// Return daemon version. No auth is required by upstream OpenACP.
    pub async fn version(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/version").await
    }

    /// Request daemon restart.
    pub async fn restart(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/restart", &Value::Object(Default::default()))
            .await
    }

    /// List registered channel adapters.
    pub async fn adapters_list(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/adapters").await
    }

    /// List OpenACP sessions.
    pub async fn sessions_list(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/sessions").await
    }

    /// Get one OpenACP session by id.
    pub async fn session_get(&self, id: &str) -> Result<OpenAcpValue, OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.get_value(&format!("/api/sessions/{id}")).await
    }

    /// Create an OpenACP session.
    pub async fn session_create(
        &self,
        req: &CreateSessionRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/sessions", req).await
    }

    /// Enqueue a prompt for a session.
    pub async fn session_prompt(
        &self,
        id: &str,
        req: &PromptRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.post_value(&format!("/api/sessions/{id}/prompt"), req)
            .await
    }

    /// Cancel a session.
    pub async fn session_cancel(&self, id: &str) -> Result<(), OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.delete_value(&format!("/api/sessions/{id}")).await
    }

    /// Enable or disable bypass permissions for a session.
    pub async fn session_bypass_set(
        &self,
        id: &str,
        req: &BypassRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.patch_value(&format!("/api/sessions/{id}/bypass"), req)
            .await
    }

    /// Resolve a pending permission request for a session.
    pub async fn session_permission_resolve(
        &self,
        id: &str,
        req: &PermissionResolveRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.post_value(&format!("/api/sessions/{id}/permission"), req)
            .await
    }

    /// Archive a session.
    pub async fn session_archive(&self, id: &str) -> Result<OpenAcpValue, OpenAcpError> {
        let id = HttpClient::encode_path_segment(id);
        self.post_value(
            &format!("/api/sessions/{id}/archive"),
            &Value::Object(Default::default()),
        )
        .await
    }

    /// Adopt an existing external agent session.
    pub async fn session_adopt(
        &self,
        req: &AdoptSessionRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/sessions/adopt", req).await
    }

    /// List configured agents.
    pub async fn agents_list(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/agents").await
    }

    /// Return upstream-redacted runtime config.
    pub async fn config_get(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/config").await
    }

    /// Return editable config metadata.
    pub async fn config_editable(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/config/editable").await
    }

    /// Patch one safe config value.
    pub async fn config_patch(
        &self,
        req: &ConfigPatchRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        self.patch_value("/api/config", req).await
    }

    /// List topics, optionally filtered by comma-separated status values.
    pub async fn topics_list(&self, status: Option<&str>) -> Result<OpenAcpValue, OpenAcpError> {
        if let Some(status) = status {
            self.get_value_query("/api/topics", &[("status".to_string(), status.to_string())])
                .await
        } else {
            self.get_value("/api/topics").await
        }
    }

    /// Delete a topic for one session.
    pub async fn topic_delete(&self, session_id: &str, force: bool) -> Result<(), OpenAcpError> {
        let session_id = HttpClient::encode_path_segment(session_id);
        if force {
            self.http
                .delete_query(
                    &format!("/api/topics/{session_id}"),
                    &[("force".to_string(), "true".to_string())],
                )
                .await?;
            Ok(())
        } else {
            self.delete_value(&format!("/api/topics/{session_id}"))
                .await
        }
    }

    /// Cleanup topics matching statuses.
    pub async fn topics_cleanup(
        &self,
        req: &TopicsCleanupRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/topics/cleanup", req).await
    }

    /// Return primary tunnel status.
    pub async fn tunnel_status(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/tunnel").await
    }

    /// List active user tunnels.
    pub async fn tunnel_list(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.get_value("/api/tunnel/list").await
    }

    /// Create a tunnel to a local port.
    pub async fn tunnel_create(
        &self,
        req: &TunnelCreateRequest,
    ) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/tunnel", req).await
    }

    /// Delete a tunnel by local port.
    pub async fn tunnel_delete(&self, port: u16) -> Result<(), OpenAcpError> {
        self.delete_value(&format!("/api/tunnel/{port}")).await
    }

    /// Delete all user tunnels.
    pub async fn tunnel_delete_all(&self) -> Result<OpenAcpValue, OpenAcpError> {
        self.http.delete("/api/tunnel").await?;
        Ok(Value::Object(Default::default()))
    }

    /// Send a notification through registered adapters.
    pub async fn notify_send(&self, req: &NotifyRequest) -> Result<OpenAcpValue, OpenAcpError> {
        self.post_value("/api/notify", req).await
    }
}
