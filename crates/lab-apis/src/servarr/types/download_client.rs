//! Download client and remote-path-mapping types.

use serde::{Deserialize, Serialize};

use super::protocol::DownloadProtocol;
use super::tag::TagId;

/// Newtype wrapper around a download-client id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DownloadClientId(pub i64);

/// Newtype wrapper around a remote-path-mapping id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemotePathMappingId(pub i64);

/// A configured download client (`SABnzbd`, `qBittorrent`, Deluge, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadClient {
    /// Client id.
    pub id: DownloadClientId,
    /// Display name.
    pub name: String,
    /// Provider implementation (e.g. `"QBittorrent"`, `"Sabnzbd"`).
    pub implementation: String,
    /// Transfer protocol this client handles.
    pub protocol: DownloadProtocol,
    /// Whether the client is enabled.
    pub enable: bool,
    /// Dispatch priority — lower runs first.
    pub priority: i32,
    /// Tags scoping which releases may use this client.
    #[serde(default)]
    pub tags: Vec<TagId>,
    /// Client-specific configuration bag.
    #[serde(default)]
    pub fields: serde_json::Value,
}

/// A remote-path mapping — translates a download-client-visible path to an
/// *arr-visible path when the two run on different hosts or containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotePathMapping {
    /// Mapping id.
    pub id: RemotePathMappingId,
    /// Hostname reported by the download client.
    pub host: String,
    /// Path as seen by the download client.
    pub remote_path: String,
    /// Path as seen by the *arr service.
    pub local_path: String,
}
