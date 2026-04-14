//! Tailscale request/response types.
//!
//! Mirrors the Tailscale API v2 spec. Field names follow the upstream schema.

use serde::{Deserialize, Serialize};

/// A device in a tailnet (node/machine).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct Device {
    /// Legacy device ID.
    pub id: String,
    /// Preferred device ID.
    pub node_id: String,
    /// `MagicDNS` name.
    pub name: String,
    /// Machine name in the admin console.
    pub hostname: String,
    /// Operating system.
    #[serde(default)]
    pub os: String,
    /// Tailscale client version.
    #[serde(default)]
    pub client_version: String,
    /// Whether a client version upgrade is available.
    #[serde(default)]
    pub update_available: bool,
    /// Tailscale IP addresses (IPv4 and IPv6).
    #[serde(default)]
    pub addresses: Vec<String>,
    /// User who registered the device.
    #[serde(default)]
    pub user: String,
    /// Tags applied to the device.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether key expiry is disabled.
    #[serde(default)]
    pub key_expiry_disabled: bool,
    /// Whether the device is authorized.
    #[serde(default)]
    pub authorized: bool,
    /// Whether the device is currently connected to the control server.
    #[serde(default)]
    pub connected_to_control: bool,
    /// When the device was last seen (ISO 8601).
    #[serde(default)]
    pub last_seen: String,
    /// When the device was created (ISO 8601).
    #[serde(default)]
    pub created: String,
    /// When the device key expires (ISO 8601).
    #[serde(default)]
    pub key_expiry: String,
    /// Whether the device is an exit node.
    #[serde(default)]
    pub is_exit_node: bool,
    /// Whether the device is ephemeral.
    #[serde(default)]
    pub is_ephemeral: bool,
}

/// Response body from `GET /tailnet/{tailnet}/devices`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

/// An auth key for adding devices to a tailnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthKey {
    /// Unique key identifier.
    pub id: String,
    /// The key string (only present on creation).
    #[serde(default)]
    pub key: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Creation timestamp.
    pub created: String,
    /// Expiry timestamp.
    #[serde(default)]
    pub expires: String,
    /// Whether the key is currently valid.
    pub invalid: bool,
    /// Capabilities granted by the key.
    pub capabilities: serde_json::Value,
}

/// Response body from `GET /tailnet/{tailnet}/keys`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyList {
    pub keys: Vec<AuthKey>,
}

/// Request body for creating an auth key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateKeyRequest {
    pub capabilities: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// DNS nameservers for a tailnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsNameservers {
    pub dns: Vec<String>,
    #[serde(rename = "magicDNS", default)]
    pub magic_dns: bool,
}

/// DNS search paths for a tailnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSearchPaths {
    pub paths: Vec<String>,
}
