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
    pub fn new(
        base_url: &str,
        auth: Auth,
        tailnet: impl Into<String>,
    ) -> Result<Self, TailscaleError> {
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

    /// Get the split DNS configuration for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn dns_split_get(&self) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .get_json(&self.tailnet_path("/dns/split-dns"))
            .await?)
    }

    /// Replace the split DNS configuration for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn dns_split_set(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .put_json(&self.tailnet_path("/dns/split-dns"), body)
            .await?)
    }

    // ── ACL / Policy ─────────────────────────────────────────────────────────

    /// Get the current ACL policy file for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn acl_get(&self) -> Result<serde_json::Value, TailscaleError> {
        Ok(self.http.get_json(&self.tailnet_path("/acl")).await?)
    }

    /// Validate an ACL policy file without applying it.
    ///
    /// Returns any validation errors from the API.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn acl_validate(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .post_json(&self.tailnet_path("/acl/validate"), body)
            .await?)
    }

    /// Set the ACL policy file for the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn acl_set(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .post_json(&self.tailnet_path("/acl"), body)
            .await?)
    }

    // ── Device extended ops ───────────────────────────────────────────────────

    /// Get advertised and accepted routes for a device.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_routes_get(
        &self,
        device_id: &str,
    ) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .get_json(&format!("/device/{device_id}/routes"))
            .await?)
    }

    /// Set the routes for a device.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_routes_set(
        &self,
        device_id: &str,
        routes: &[String],
    ) -> Result<serde_json::Value, TailscaleError> {
        let body = serde_json::json!({ "routes": routes });
        Ok(self
            .http
            .post_json(&format!("/device/{device_id}/routes"), &body)
            .await?)
    }

    /// Set tags for a device.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_tag(&self, device_id: &str, tags: &[String]) -> Result<(), TailscaleError> {
        let body = serde_json::json!({ "tags": tags });
        Ok(self
            .http
            .post_json::<_, serde_json::Value>(&format!("/device/{device_id}/tags"), &body)
            .await
            .map(|_| ())?)
    }

    /// Expire the key for a device, forcing re-authentication.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn device_expire(&self, device_id: &str) -> Result<(), TailscaleError> {
        let body = serde_json::json!({});
        Ok(self
            .http
            .post_json::<_, serde_json::Value>(&format!("/device/{device_id}/expire"), &body)
            .await
            .map(|_| ())?)
    }

    // ── Users ─────────────────────────────────────────────────────────────────

    /// List all users in the tailnet.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn user_list(&self) -> Result<serde_json::Value, TailscaleError> {
        Ok(self.http.get_json(&self.tailnet_path("/users")).await?)
    }

    // ── Tailnet Settings ──────────────────────────────────────────────────────

    /// Get tailnet-level settings.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn tailnet_settings_get(&self) -> Result<serde_json::Value, TailscaleError> {
        Ok(self.http.get_json(&self.tailnet_path("/settings")).await?)
    }

    /// Patch tailnet-level settings.
    ///
    /// # Errors
    /// Returns `TailscaleError::Api` on HTTP failure.
    pub async fn tailnet_settings_patch(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, TailscaleError> {
        Ok(self
            .http
            .patch_json(&self.tailnet_path("/settings"), body)
            .await?)
    }
}
