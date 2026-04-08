//! Indexer configuration types.

use serde::{Deserialize, Serialize};

use super::protocol::DownloadProtocol;
use super::tag::TagId;

/// Newtype wrapper around an indexer id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IndexerId(pub i64);

/// An indexer configured in an *arr service (typically managed by Prowlarr).
///
/// Mirrors `IndexerResource` from the Radarr v3 / Sonarr v3 / Prowlarr v1
/// `OpenAPI` specs. Only the display-relevant fields are modeled; the full
/// `fields` array (arbitrary key/value settings) is kept as raw JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Indexer {
    /// Indexer id.
    pub id: IndexerId,
    /// Display name.
    pub name: String,
    /// Provider implementation.
    pub implementation: String,
    /// Transfer protocol this indexer serves.
    pub protocol: DownloadProtocol,
    /// Whether the indexer is enabled.
    pub enable: bool,
    /// Dispatch priority — lower runs first.
    pub priority: i32,
    /// Tag ids scoping which resources may use this indexer.
    #[serde(default)]
    pub tags: Vec<TagId>,
    /// Indexer-specific configuration bag.
    #[serde(default)]
    pub fields: serde_json::Value,
}
