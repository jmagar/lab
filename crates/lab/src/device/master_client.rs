use anyhow::{Context, Result};

use crate::config::LabConfig;
use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::identity::resolve_local_hostname;

#[derive(Debug, Clone)]
pub struct MasterClient {
    http: reqwest::Client,
    base_url: String,
}

impl MasterClient {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
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
        self.get_json(&format!("/v1/device/devices/{device_id}")).await
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
        let host = match config.device.as_ref().and_then(|prefs| prefs.master.as_deref()) {
            Some(host) => host.to_string(),
            None => resolve_local_hostname()?,
        };
        let port = config.mcp.port.unwrap_or(8765);
        Ok(Self::new(format!("http://{host}:{port}")))
    }

    async fn post_json<T: serde::Serialize + ?Sized>(&self, path: &str, payload: &T) -> Result<()> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        self.http
            .post(&url)
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
            .http
            .get(&url)
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
            .http
            .post(&url)
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
}
