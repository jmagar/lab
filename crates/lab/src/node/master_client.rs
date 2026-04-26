use anyhow::Result;
use lab_apis::core::Auth;
use lab_apis::device_runtime::client::NodeRuntimeClient;

use crate::config::LabConfig;
use crate::node::identity::resolve_local_hostname;

#[derive(Debug, Clone)]
pub struct MasterClient {
    base_url: String,
    inner: NodeRuntimeClient,
}

impl MasterClient {
    #[allow(dead_code)]
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Self::with_bearer_token(base_url, None)
    }

    pub fn with_bearer_token(
        base_url: impl Into<String>,
        bearer_token: Option<String>,
    ) -> Result<Self> {
        let base_url = base_url.into();
        let auth = bearer_token.map_or(Auth::None, |token| Auth::Bearer { token });
        let inner = NodeRuntimeClient::new(base_url.clone(), auth)?;
        Ok(Self { base_url, inner })
    }

    pub async fn fetch_devices(&self) -> Result<serde_json::Value> {
        self.inner.fetch_devices().await.map_err(Into::into)
    }

    pub async fn fetch_device(&self, node_id: &str) -> Result<serde_json::Value> {
        self.inner.fetch_device(node_id).await.map_err(Into::into)
    }

    pub async fn fetch_enrollments(&self) -> Result<serde_json::Value> {
        self.inner.fetch_enrollments().await.map_err(Into::into)
    }

    pub async fn approve_enrollment(
        &self,
        node_id: &str,
        note: Option<&str>,
    ) -> Result<serde_json::Value> {
        self.inner
            .approve_enrollment(node_id, note)
            .await
            .map_err(Into::into)
    }

    pub async fn deny_enrollment(
        &self,
        node_id: &str,
        reason: Option<&str>,
    ) -> Result<serde_json::Value> {
        self.inner
            .deny_enrollment(node_id, reason)
            .await
            .map_err(Into::into)
    }

    pub async fn post_log_ingest(&self, payload: &serde_json::Value) -> Result<serde_json::Value> {
        self.inner
            .post_log_ingest(payload)
            .await
            .map_err(Into::into)
    }

    pub async fn search_logs(&self, node_id: &str, query: &str) -> Result<serde_json::Value> {
        self.inner
            .search_logs(node_id, query)
            .await
            .map_err(Into::into)
    }

    pub fn from_config(config: &LabConfig, port_override: Option<u16>) -> Result<Self> {
        let host = match config.controller_host() {
            Some(host) => host.to_string(),
            None => resolve_local_hostname()?,
        };
        let port = port_override.or(config.mcp.port).unwrap_or(8765);
        Self::with_bearer_token(format!("http://{host}:{port}"), master_bearer_token())
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

fn master_bearer_token() -> Option<String> {
    std::env::var("LAB_MCP_HTTP_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
}
