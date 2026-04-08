//! Tag types.

use serde::{Deserialize, Serialize};

use super::download_client::DownloadClientId;
use super::indexer::IndexerId;
use super::notification::NotificationId;

/// Newtype wrapper around a tag id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagId(pub i64);

/// A free-form label attached to resources (movies, series, indexers, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Tag id.
    pub id: TagId,
    /// Human-readable label.
    pub label: String,
}

/// A tag plus the ids of every resource currently carrying it.
///
/// Mirrors `TagDetailsResource` from the Radarr v3 / Sonarr v3 OpenAPI specs.
/// The per-service resource-id array (`movie_ids` on radarr, `series_ids` on
/// sonarr) is kept separately on each service; the cross-service arrays
/// (indexers, download clients, notifications) live here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDetail {
    /// Tag id.
    pub id: TagId,
    /// Human-readable label.
    pub label: String,
    /// Indexers carrying this tag.
    #[serde(default)]
    pub indexer_ids: Vec<IndexerId>,
    /// Download clients carrying this tag.
    #[serde(default)]
    pub download_client_ids: Vec<DownloadClientId>,
    /// Notification providers carrying this tag.
    #[serde(default)]
    pub notification_ids: Vec<NotificationId>,
}
