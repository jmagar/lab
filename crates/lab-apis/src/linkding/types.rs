//! Linkding request/response types.

use serde::{Deserialize, Serialize};

/// Query parameters for listing bookmarks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkListParams {
    /// Filter by search phrase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    /// Maximum number of results (default 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Index from which to start returning results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Payload for creating a new bookmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkWriteRequest {
    /// Bookmark URL (required).
    pub url: String,
    /// Optional title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional markdown notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Save directly to archive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    /// Mark as unread.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread: Option<bool>,
    /// Share publicly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    /// Tag names to assign.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
}

/// Payload for partially updating an existing bookmark (PATCH).
///
/// All fields are optional — only provided fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkUpdateRequest {
    /// New URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// New title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Update unread status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread: Option<bool>,
    /// Update shared status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    /// Replace tag names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
}

/// Payload for creating a new tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCreateRequest {
    /// Tag name.
    pub name: String,
}

// ── Bundles ───────────────────────────────────────────────────────────────────

/// A Linkding bundle (saved search / smart collection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    /// Bundle ID.
    pub id: i64,
    /// Bundle name.
    pub name: String,
    /// Linkding search query that populates this bundle.
    pub search_query: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload for creating a new bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleCreate {
    /// Bundle name.
    pub name: String,
    /// Linkding search query.
    pub search_query: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload for partially updating a bundle (PATCH).
///
/// All fields are optional — only provided fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleUpdate {
    /// New name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New search query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_query: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Bookmark Assets ───────────────────────────────────────────────────────────

/// An asset attached to a bookmark (snapshot, PDF, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkAsset {
    /// Asset ID.
    pub id: i64,
    /// Asset type: `"snapshot"` or `"pdf"`.
    pub asset_type: String,
    /// Processing status: `"pending"`, `"complete"`, or `"failure"`.
    pub status: String,
    /// File size in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// ISO-8601 creation timestamp.
    pub date_created: String,
}
