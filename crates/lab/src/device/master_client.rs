use anyhow::{Context, Result};

use crate::device::checkin::{DeviceHello, DeviceStatus};

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

    pub async fn post_metadata(&self, payload: &serde_json::Value) -> Result<()> {
        self.post_json("/v1/device/metadata", payload).await
    }

    pub async fn post_syslog_batch(&self, payload: &serde_json::Value) -> Result<()> {
        self.post_json("/v1/device/syslog/batch", payload).await
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
}
