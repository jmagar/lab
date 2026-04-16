use anyhow::{Context, Result};

use crate::config::LabConfig;
use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::identity::resolve_local_hostname;

const DEFAULT_MASTER_CLIENT_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Clone)]
pub struct MasterClient {
    http: reqwest::Client,
    base_url: String,
    bearer_token: Option<String>,
}

impl MasterClient {
    #[must_use]
    #[allow(dead_code)]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_bearer_token(base_url, None)
    }

    #[must_use]
    pub fn with_bearer_token(base_url: impl Into<String>, bearer_token: Option<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(
                    DEFAULT_MASTER_CLIENT_TIMEOUT_SECS,
                ))
                .build()
                .expect("master client builder should be valid"),
            base_url: base_url.into(),
            bearer_token,
        }
    }

    pub async fn post_hello(&self, payload: &DeviceHello) -> Result<()> {
        self.post_json("/v1/device/hello", payload).await
    }

    pub async fn post_status(&self, payload: &DeviceStatus) -> Result<()> {
        self.post_json("/v1/device/status", payload).await
    }

    pub async fn post_metadata(&self, payload: &DeviceMetadataUpload) -> Result<()> {
        self.post_json("/v1/device/metadata", payload).await
    }

    pub async fn post_syslog_batch(&self, payload: &serde_json::Value) -> Result<()> {
        self.post_json("/v1/device/syslog/batch", payload).await
    }

    pub async fn fetch_devices(&self) -> Result<serde_json::Value> {
        self.get_json("/v1/device/devices").await
    }

    pub async fn fetch_device(&self, device_id: &str) -> Result<serde_json::Value> {
        self.get_json(&format!("/v1/device/devices/{device_id}"))
            .await
    }

    pub async fn search_logs(&self, device_id: &str, query: &str) -> Result<serde_json::Value> {
        self.post_json_value(
            "/v1/device/logs/search",
            &serde_json::json!({
                "device_id": device_id,
                "query": query,
            }),
        )
        .await
    }

    pub fn from_config(config: &LabConfig) -> Result<Self> {
        let host = match config
            .device
            .as_ref()
            .and_then(|prefs| prefs.master.as_deref())
        {
            Some(host) => host.to_string(),
            None => resolve_local_hostname()?,
        };
        let port = config.mcp.port.unwrap_or(8765);
        Ok(Self::with_bearer_token(
            format!("http://{host}:{port}"),
            master_bearer_token(),
        ))
    }

    async fn post_json<T: serde::Serialize + ?Sized>(&self, path: &str, payload: &T) -> Result<()> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        self.request(self.http.post(&url))
            .json(payload)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?
            .error_for_status()
            .with_context(|| format!("POST {url} failed"))?;
        Ok(())
    }

    async fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let response = self
            .request(self.http.get(&url))
            .send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("GET {url} failed"))?;
        response
            .json()
            .await
            .with_context(|| format!("decode {url}"))
    }

    async fn post_json_value<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        payload: &T,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let response = self
            .request(self.http.post(&url))
            .json(payload)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?
            .error_for_status()
            .with_context(|| format!("POST {url} failed"))?;
        response
            .json()
            .await
            .with_context(|| format!("decode {url}"))
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.bearer_token.as_deref() {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }
}

fn master_bearer_token() -> Option<String> {
    std::env::var("LAB_MCP_HTTP_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
}
