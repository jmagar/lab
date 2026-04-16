use std::time::Duration;

use crate::core::{ApiError, Auth, HttpClient};

#[derive(Debug, Clone)]
pub struct DeviceRuntimeClient {
    http: HttpClient,
}

impl DeviceRuntimeClient {
    /// Build a client against the device-runtime surface with an optional request timeout.
    ///
    /// # Errors
    /// Returns [`ApiError`] if the shared HTTP client cannot be built.
    pub fn new(
        base_url: impl Into<String>,
        auth: Auth,
        timeout: Option<Duration>,
    ) -> Result<Self, ApiError> {
        let http = match timeout {
            Some(timeout) => HttpClient::with_default_headers_and_timeouts(
                base_url,
                auth,
                reqwest::header::HeaderMap::new(),
                Duration::from_secs(5),
                timeout,
            )?,
            None => HttpClient::new(base_url, auth)?,
        };
        Ok(Self { http })
    }

    pub async fn post_hello<T: serde::Serialize + Sync>(&self, payload: &T) -> Result<(), ApiError> {
        self.http.post_void("/v1/device/hello", payload).await
    }

    pub async fn post_status<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.http.post_void("/v1/device/status", payload).await
    }

    pub async fn post_metadata<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.http.post_void("/v1/device/metadata", payload).await
    }

    pub async fn post_syslog_batch<T: serde::Serialize + Sync>(
        &self,
        payload: &T,
    ) -> Result<(), ApiError> {
        self.http.post_void("/v1/device/syslog/batch", payload).await
    }

    pub async fn fetch_devices(&self) -> Result<serde_json::Value, ApiError> {
        self.http.get_json("/v1/device/devices").await
    }

    pub async fn fetch_device(&self, device_id: &str) -> Result<serde_json::Value, ApiError> {
        self.http
            .get_json(&format!("/v1/device/devices/{device_id}"))
            .await
    }

    pub async fn search_logs(
        &self,
        device_id: &str,
        query: &str,
    ) -> Result<serde_json::Value, ApiError> {
        self.http
            .post_json(
                "/v1/device/logs/search",
                &serde_json::json!({
                    "device_id": device_id,
                    "query": query,
                }),
            )
            .await
    }
}
