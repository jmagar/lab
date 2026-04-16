use std::future::Future;
use std::time::Duration;

use super::types::SearchLogsRequest;
use crate::core::{ApiError, Auth, HttpClient};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

const DEVICE_RUNTIME_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct DeviceRuntimeClient {
    http: HttpClient,
}

impl DeviceRuntimeClient {
    /// Build a client against the device-runtime surface.
    ///
    /// # Errors
    /// Returns [`ApiError`] if the shared HTTP client cannot be built.
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Result<Self, ApiError> {
        let http = HttpClient::new(base_url, auth)?;
        Ok(Self { http })
    }

    pub async fn post_hello<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.with_timeout(self.http.post_void("/v1/device/hello", payload))
            .await
    }

    pub async fn post_status<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.with_timeout(self.http.post_void("/v1/device/status", payload))
            .await
    }

    pub async fn post_metadata<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.with_timeout(self.http.post_void("/v1/device/metadata", payload))
            .await
    }

    pub async fn post_syslog_batch<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.with_timeout(self.http.post_void("/v1/device/syslog/batch", payload))
            .await
    }

    pub async fn fetch_devices(&self) -> Result<serde_json::Value, ApiError> {
        self.with_timeout(self.http.get_json("/v1/device/devices"))
            .await
    }

    pub async fn fetch_device(&self, device_id: &str) -> Result<serde_json::Value, ApiError> {
        let encoded_id = utf8_percent_encode(device_id, NON_ALPHANUMERIC).to_string();
        self.with_timeout(
            self.http
                .get_json(&format!("/v1/device/devices/{encoded_id}")),
        )
        .await
    }

    pub async fn search_logs(
        &self,
        device_id: &str,
        query: &str,
    ) -> Result<serde_json::Value, ApiError> {
        let request = SearchLogsRequest {
            device_id: device_id.to_string(),
            query: query.to_string(),
        };
        self.with_timeout(self.http.post_json("/v1/device/logs/search", &request))
            .await
    }

    async fn with_timeout<T>(
        &self,
        future: impl Future<Output = Result<T, ApiError>>,
    ) -> Result<T, ApiError> {
        tokio::time::timeout(DEVICE_RUNTIME_TIMEOUT, future)
            .await
            .map_err(|_| ApiError::Network("request timed out".to_string()))?
    }
}
