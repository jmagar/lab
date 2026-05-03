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

    /// Returns `true` when the controller reports an active WebSocket connection
    /// for the given node. Returns `false` if the node is known but not connected,
    /// or if the node is not in inventory.
    ///
    /// Unlike `fetch_device`, this returns `false` on node-not-found rather than
    /// an error — it is intended as a polling helper for rollout verification, not
    /// as an inventory query.
    pub async fn node_connected(&self, node_id: &str) -> Result<bool> {
        match self.fetch_device(node_id).await {
            Ok(value) => Ok(value
                .get("connected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)),
            Err(error) => {
                // If the node is simply not found, it's not connected.
                let msg = error.to_string();
                if msg.contains("not_found") || msg.contains("404") {
                    return Ok(false);
                }
                Err(error)
            }
        }
    }

    /// Poll `node_connected` until the node is connected or the timeout elapses.
    ///
    /// Returns `Ok(())` on success.
    /// Returns `Err` when the timeout expires without a successful `true` response.
    ///
    /// Transport errors are logged as warnings and treated as "not yet connected"
    /// — they will not abort the poll early.
    pub async fn wait_for_node_connected(
        &self,
        node_id: &str,
        timeout: std::time::Duration,
    ) -> Result<()> {
        use std::time::Instant;
        let deadline = Instant::now() + timeout;
        let mut attempt: u32 = 0;
        loop {
            match self.node_connected(node_id).await {
                Ok(true) => return Ok(()),
                Ok(false) => {}
                Err(error) => {
                    tracing::warn!(
                        surface = "node",
                        service = "update",
                        action = "node_connected.poll",
                        node_id = %node_id,
                        attempt,
                        error = %error,
                        "node_connected poll returned error",
                    );
                }
            }
            if Instant::now() >= deadline {
                anyhow::bail!(
                    "timed out waiting for node `{node_id}` to reconnect to controller ({}s)",
                    timeout.as_secs()
                );
            }
            attempt += 1;
            // Exponential backoff: 2s, 4s, 8s, capped at 16s.
            let delay =
                std::time::Duration::from_secs(std::cmp::min(2u64.saturating_pow(attempt), 16));
            tokio::time::sleep(delay).await;
        }
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
