//! `TailscaleClient` — async methods against the Tailscale API v2.
//!
//! Authentication: Bearer token (`tskey-api-*`).
//! Base URL: `https://api.tailscale.com/api/v2`
//!
//! All paths are relative to the base URL. The `tailnet` parameter is typically
//! the domain name of the tailnet (e.g. `example.com`) or the special value `-`
//! to use the default tailnet for the authenticated key.

use crate::core::{Auth, HttpClient};

use super::error::TailscaleError;
use super::types::{
    AuthKey, CreateKeyRequest, Device, DeviceList, DnsNameservers, DnsSearchPaths, KeyList,
};

/// Client for the Tailscale API v2.
pub struct TailscaleClient {
    http: HttpClient,
    /// Tailnet to use in paths. Usually `-` (default for the auth key) or `example.com`.
    tailnet: String,
}

impl TailscaleClient {
    /// Build a client with bearer token auth.
    ///
    /// # Errors
    /// Returns [`TailscaleError::Api`] if TLS initialization fails.
    pub fn new(base_url: &str, auth: Auth, tailnet: impl Into<String>) -> Result<Self, TailscaleError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            tailnet: tailnet.into(),
        })
    }

    fn tailnet_path(&self, suffix: &str) -> String {
        format!("/tailnet/{}{suffix}", self.tailnet)
    }

    // ── Devices ─────────────────────────────────────────────────────────────

    /// List all devices in the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn devices_list(&self) -> Result<DeviceList, TailscaleError> {
        Ok(self.http.get_json(&self.tailnet_path("/devices")).await?)
    }

    /// Get a single device by ID.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_get(&self, device_id: &str) -> Result<Device, TailscaleError> {
        Ok(self.http.get_json(&format!("/device/{device_id}")).await?)
    }

    /// Delete a device from the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_delete(&self, device_id: &str) -> Result<(), TailscaleError> {
        Ok(self.http.delete(&format!("/device/{device_id}")).await?)
    }

    /// Authorize or de-authorize a device.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_authorize(
        &self,
        device_id: &str,
        authorized: bool,
    ) -> Result<(), TailscaleError> {
        let body = serde_json::json!({ "authorized": authorized });
        Ok(self
            .http
            .post_json::<_, serde_json::Value>(&format!("/device/{device_id}/authorized"), &body)
            .await
            .map(|_| ())?)
    }

    // ── Auth Keys ────────────────────────────────────────────────────────────

    /// List all auth keys for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn keys_list(&self) -> Result<KeyList, TailscaleError> {
        Ok(self.http.get_json(&self.tailnet_path("/keys")).await?)
    }

    /// Create a new auth key.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn key_create(&self, req: &CreateKeyRequest) -> Result<AuthKey, TailscaleError> {
        Ok(self
            .http
            .post_json(&self.tailnet_path("/keys"), req)
            .await?)
    }

    /// Get a specific auth key.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn key_get(&self, key_id: &str) -> Result<AuthKey, TailscaleError> {
        Ok(self
            .http
            .get_json(&self.tailnet_path(&format!("/keys/{key_id}")))
            .await?)
    }

    /// Delete an auth key.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn key_delete(&self, key_id: &str) -> Result<(), TailscaleError> {
        Ok(self
            .http
            .delete(&self.tailnet_path(&format!("/keys/{key_id}")))
            .await?)
    }

    // ── DNS ──────────────────────────────────────────────────────────────────

    /// Get the DNS nameservers for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn dns_nameservers(&self) -> Result<DnsNameservers, TailscaleError> {
        Ok(self
            .http
            .get_json(&self.tailnet_path("/dns/nameservers"))
            .await?)
    }

    /// Get the DNS search paths for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn dns_search_paths(&self) -> Result<DnsSearchPaths, TailscaleError> {
        Ok(self
            .http
            .get_json(&self.tailnet_path("/dns/searchpaths"))
            .await?)
    }
}
