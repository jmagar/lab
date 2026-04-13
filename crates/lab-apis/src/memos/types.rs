//! Memos request/response types.
//!
//! Targets the **Memos v1 API** (resource-name style, RFC3339 timestamps).

use serde::{Deserialize, Serialize};

/// A Memos memo entry (Memos v1 API shape).
///
/// The v1 API uses resource-name identifiers (`"memos/123"`) and RFC3339
/// timestamps — not the legacy integer `id` / Unix `created_ts` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    /// Resource name in the form `"memos/{id}"`.
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// RFC3339 creation timestamp.
    #[serde(rename = "createTime")]
    pub create_time: String,
    /// RFC3339 last-update timestamp.
    #[serde(rename = "updateTime")]
    pub update_time: String,
    pub visibility: String,
}

/// Query parameters for listing memos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListMemosParams {
    /// Filter to memos by creator resource name (e.g. `"users/1"`).
    pub creator: Option<String>,
    /// Filter to memos by tag name.
    pub tag: Option<String>,
    /// Maximum number of memos to return per page.
    pub page_size: Option<u32>,
    /// Page token for continuing a previous list request.
    pub page_token: Option<String>,
}

/// Request body for creating a memo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoRequest {
    /// Markdown content of the memo.
    pub content: String,
    /// Visibility: `"PRIVATE"`, `"PROTECTED"`, or `"PUBLIC"`.
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "PRIVATE".to_string()
}

/// Request body for updating a memo (PATCH — only provided fields are changed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoRequest {
    /// New markdown content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// New visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}
