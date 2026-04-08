//! `UnifiClient` — read-only methods for the `UniFi` Network integration API.
//!
//! The integration API lives under `/proxy/network/integration/v1`. The
//! client keeps the endpoint prefix internal so callers only provide the
//! controller root URL.

use serde_json::Value;

use crate::core::{Auth, HttpClient};

use super::{UnifiError, types::ApplicationInfo};

/// Client for a `UniFi` Network Application controller.
pub struct UnifiClient {
    pub(crate) http: HttpClient,
}

impl UnifiClient {
    /// Build a client against the controller root URL.
    ///
    /// `UniFi` uses API-key auth with the `X-API-KEY` header:
    /// `Auth::ApiKey { header: "X-API-KEY".into(), key: <api_key> }`.
    ///
    /// # Errors
    /// Returns [`UnifiError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UnifiError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    fn path(path: &str) -> String {
        format!("/proxy/network/integration/v1{path}")
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, UnifiError> {
        self.http
            .get_json(&Self::path(path))
            .await
            .map_err(Into::into)
    }

    async fn get_json_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T, UnifiError> {
        self.http
            .get_json_query(&Self::path(path), query)
            .await
            .map_err(Into::into)
    }

    async fn post_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, UnifiError> {
        self.http
            .post_json(&Self::path(path), body)
            .await
            .map_err(Into::into)
    }

    async fn put_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, UnifiError> {
        self.http
            .put_json(&Self::path(path), body)
            .await
            .map_err(Into::into)
    }

    async fn patch_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, UnifiError> {
        self.http
            .patch_json(&Self::path(path), body)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, path: &str) -> Result<(), UnifiError> {
        self.http
            .delete(&Self::path(path))
            .await
            .map_err(Into::into)
    }

    async fn delete_query(&self, path: &str, query: &[(String, String)]) -> Result<(), UnifiError> {
        self.http
            .delete_query(&Self::path(path), query)
            .await
            .map_err(Into::into)
    }

    /// Return application version and runtime metadata.
    pub async fn info(&self) -> Result<ApplicationInfo, UnifiError> {
        self.get_json("/info").await
    }

    /// List local sites.
    pub async fn sites_list(&self) -> Result<Value, UnifiError> {
        self.get_json("/sites").await
    }

    /// List adopted devices for one site.
    pub async fn devices_list(&self, site_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/devices")).await
    }

    /// Inspect one adopted device.
    pub async fn device_get(&self, site_id: &str, device_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/devices/{device_id}"))
            .await
    }

    /// Get latest statistics for one adopted device.
    pub async fn device_stats_latest(
        &self,
        site_id: &str,
        device_id: &str,
    ) -> Result<Value, UnifiError> {
        self.get_json(&format!(
            "/sites/{site_id}/devices/{device_id}/statistics/latest"
        ))
        .await
    }

    /// List pending adoption devices.
    pub async fn pending_devices_list(&self) -> Result<Value, UnifiError> {
        self.get_json("/pending-devices").await
    }

    /// List connected clients for one site.
    pub async fn clients_list(&self, site_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/clients")).await
    }

    /// Inspect one connected client.
    pub async fn client_get(&self, site_id: &str, client_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/clients/{client_id}"))
            .await
    }

    /// List networks for one site.
    pub async fn networks_list(&self, site_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/networks")).await
    }

    /// Inspect one network.
    pub async fn network_get(&self, site_id: &str, network_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/networks/{network_id}"))
            .await
    }

    /// Get references for one network.
    pub async fn network_references(
        &self,
        site_id: &str,
        network_id: &str,
    ) -> Result<Value, UnifiError> {
        self.get_json(&format!(
            "/sites/{site_id}/networks/{network_id}/references"
        ))
        .await
    }

    /// List `WiFi` broadcasts for one site.
    pub async fn wifi_broadcasts_list(&self, site_id: &str) -> Result<Value, UnifiError> {
        self.get_json(&format!("/sites/{site_id}/wifi/broadcasts"))
            .await
    }

    /// Inspect one `WiFi` broadcast.
    pub async fn wifi_broadcast_get(
        &self,
        site_id: &str,
        wifi_broadcast_id: &str,
    ) -> Result<Value, UnifiError> {
        self.get_json(&format!(
            "/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"
        ))
        .await
    }

    /// Generic JSON GET helper for the remaining endpoints.
    pub async fn get_value(&self, path: &str) -> Result<Value, UnifiError> {
        self.get_json(path).await
    }

    /// Generic JSON GET helper with query parameters.
    pub async fn get_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Value, UnifiError> {
        self.get_json_query(path, query).await
    }

    /// Generic JSON POST helper.
    pub async fn post_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, UnifiError> {
        self.post_json(path, body).await
    }

    /// Generic JSON PUT helper.
    pub async fn put_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, UnifiError> {
        self.put_json(path, body).await
    }

    /// Generic JSON PATCH helper.
    pub async fn patch_value<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Value, UnifiError> {
        self.patch_json(path, body).await
    }

    /// Generic JSON DELETE helper.
    pub async fn delete_value(&self, path: &str) -> Result<(), UnifiError> {
        self.delete(path).await
    }

    /// Generic JSON DELETE helper with query parameters.
    pub async fn delete_value_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<(), UnifiError> {
        self.delete_query(path, query).await
    }
}
