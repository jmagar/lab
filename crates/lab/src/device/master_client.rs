use anyhow::Result;
use lab_apis::core::Auth;
use lab_apis::device_runtime::client::DeviceRuntimeClient;

use crate::config::LabConfig;
use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::identity::resolve_local_hostname;

#[derive(Debug, Clone)]
pub struct MasterClient {
    inner: DeviceRuntimeClient,
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
        let auth = bearer_token.map_or(Auth::None, |token| Auth::Bearer { token });
        let inner = DeviceRuntimeClient::new(base_url, auth)?;
        Ok(Self { inner })
    }

    pub async fn post_hello(&self, payload: &DeviceHello) -> Result<()> {
        self.inner.post_hello(payload).await.map_err(Into::into)
    }

    pub async fn post_status(&self, payload: &DeviceStatus) -> Result<()> {
        self.inner.post_status(payload).await.map_err(Into::into)
    }

    pub async fn post_metadata(&self, payload: &DeviceMetadataUpload) -> Result<()> {
        self.inner.post_metadata(payload).await.map_err(Into::into)
    }

    pub async fn post_syslog_batch(&self, payload: &serde_json::Value) -> Result<()> {
        self.inner
            .post_syslog_batch(payload)
            .await
            .map_err(Into::into)
    }

    pub async fn fetch_devices(&self) -> Result<serde_json::Value> {
        self.inner.fetch_devices().await.map_err(Into::into)
    }

    pub async fn fetch_device(&self, device_id: &str) -> Result<serde_json::Value> {
        self.inner.fetch_device(device_id).await.map_err(Into::into)
    }

    pub async fn search_logs(&self, device_id: &str, query: &str) -> Result<serde_json::Value> {
        self.inner
            .search_logs(device_id, query)
            .await
            .map_err(Into::into)
    }

    pub fn from_config(config: &LabConfig, port_override: Option<u16>) -> Result<Self> {
        let host = match config
            .device
            .as_ref()
            .and_then(|prefs| prefs.master.as_deref())
        {
            Some(host) => host.to_string(),
            None => resolve_local_hostname()?,
        };
        let port = port_override.or(config.mcp.port).unwrap_or(8765);
        Self::with_bearer_token(format!("http://{host}:{port}"), master_bearer_token())
    }
}

fn master_bearer_token() -> Option<String> {
    std::env::var("LAB_MCP_HTTP_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
}
