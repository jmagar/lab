//! Plex request/response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Plex library section (Movies, TV Shows, Music, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySection {
    pub key: String,
    pub title: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub agent: String,
    pub scanner: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub uuid: String,
}

/// Container for library sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LibrarySectionsContainer {
    pub media_container: LibrarySectionsMediaContainer,
}

/// Inner media container for library sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LibrarySectionsMediaContainer {
    pub size: i64,
    pub directory: Vec<Value>,
}

/// Parameters for media/library search.
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<u32>,
    pub section_id: Option<String>,
}

/// Parameters for session termination.
#[derive(Debug, Clone)]
pub struct TerminateSessionParams {
    pub session_id: String,
    pub reason: Option<String>,
}
