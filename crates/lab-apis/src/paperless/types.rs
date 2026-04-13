//! Paperless-ngx request/response types.

use serde::{Deserialize, Serialize};

/// Partial update request for a document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentUpdateRequest {
    /// Document title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// ISO-8601 creation date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Correspondent ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correspondent: Option<u64>,
    /// Document type ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<u64>,
    /// Tag IDs to assign.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<u64>>,
}

/// Request body to create a tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCreateRequest {
    /// Tag name.
    pub name: String,
    /// Hex color string (e.g. `"#ff0000"`).
    #[serde(rename = "color", skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    /// Whether this is the special inbox tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_inbox_tag: Option<bool>,
    /// Matching expression (for auto-tagging rules).
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_expr: Option<String>,
    /// Matching algorithm ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_algorithm: Option<u32>,
}

/// Request body to create a correspondent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrespondentCreateRequest {
    /// Correspondent name.
    pub name: String,
    /// Matching expression (for auto-assign rules).
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_expr: Option<String>,
    /// Matching algorithm ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_algorithm: Option<u32>,
}

/// Request body to create a document type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTypeCreateRequest {
    /// Document type name.
    pub name: String,
    /// Matching expression (for auto-classify rules).
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_expr: Option<String>,
    /// Matching algorithm ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_algorithm: Option<u32>,
}
