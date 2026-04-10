//! Linkding request/response types.
//!
//! TODO: add tag and paginated list types from the Linkding API spec.

use serde::{Deserialize, Serialize};

/// A Linkding bookmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub tag_names: Vec<String>,
    pub is_archived: bool,
    pub date_added: String,
    pub date_modified: String,
}
