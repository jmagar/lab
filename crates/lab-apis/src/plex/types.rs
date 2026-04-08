//! Plex request/response types.
//!
//! TODO: add library section, media item, session, and device types from the Plex HTTP API spec.

use serde::{Deserialize, Serialize};

/// A Plex library section (Movies, TV Shows, Music, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySection {
    pub key: String,
    pub title: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub agent: String,
    pub scanner: String,
}
