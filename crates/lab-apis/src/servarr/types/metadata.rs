//! Metadata writer types.

use serde::{Deserialize, Serialize};

use super::tag::TagId;

/// Newtype wrapper around a metadata-writer id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MetadataId(pub i64);

/// A configured metadata writer (Kodi NFO, Wdtv, Plex matroska, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Writer id.
    pub id: MetadataId,
    /// Display name.
    pub name: String,
    /// Provider implementation.
    pub implementation: String,
    /// Whether the writer is enabled.
    pub enable: bool,
    /// Tag ids scoping which resources trigger metadata writes.
    #[serde(default)]
    pub tags: Vec<TagId>,
    /// Writer-specific configuration bag.
    #[serde(default)]
    pub fields: serde_json::Value,
}
