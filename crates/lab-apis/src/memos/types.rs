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

// ── Attachments ───────────────────────────────────────────────────────────────

/// An uploaded attachment returned by `POST /api/v1/attachments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Resource name, e.g. `"attachments/123"`.
    pub name: String,
    /// Original filename.
    pub filename: String,
    /// MIME type.
    #[serde(rename = "type")]
    pub content_type: String,
    /// File size in bytes.
    pub size: Option<i64>,
    /// External URL for the attachment (if stored remotely).
    #[serde(rename = "externalLink")]
    pub external_link: Option<String>,
}

// ── Comments ─────────────────────────────────────────────────────────────────

/// A comment on a memo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Resource name, e.g. `"memos/456/comments/789"`.
    pub name: String,
    /// Comment content.
    pub content: String,
    /// RFC3339 creation timestamp.
    #[serde(rename = "createTime")]
    pub create_time: String,
    /// RFC3339 update timestamp.
    #[serde(rename = "updateTime")]
    pub update_time: String,
}

/// Request body for creating a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    /// Comment content.
    pub content: String,
}

// ── Share links ───────────────────────────────────────────────────────────────

/// A share link for a memo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    /// Resource name, e.g. `"memos/123/shares/abc"`.
    pub name: String,
    /// RFC3339 creation timestamp.
    #[serde(rename = "createTime")]
    pub create_time: String,
}

// ── User stats ────────────────────────────────────────────────────────────────

/// Statistics for a user's memo activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    /// Total memo count for the user.
    #[serde(rename = "memoCount")]
    pub memo_count: Option<i64>,
    /// All tags used by the user with counts.
    #[serde(default)]
    pub tags: std::collections::HashMap<String, i64>,
}

// ── Webhooks ──────────────────────────────────────────────────────────────────

/// A webhook registered for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Resource name, e.g. `"users/1/webhooks/42"`.
    pub name: String,
    /// Display name for the webhook.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// Webhook URL to POST events to.
    pub url: String,
    /// RFC3339 creation timestamp.
    #[serde(rename = "createTime")]
    pub create_time: String,
}

/// Request body for creating a webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    /// Webhook URL to POST events to.
    pub url: String,
    /// Display name for the webhook.
    #[serde(rename = "displayName")]
    pub display_name: String,
}

// ── MemoUser ──────────────────────────────────────────────────────────────────

/// A user entry returned by `GET /api/v1/users`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoUser {
    /// Resource name, e.g. `"users/1"`.
    pub name: String,
    /// Username / handle.
    pub username: Option<String>,
    /// Display name.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Role: `"HOST"`, `"ADMIN"`, `"USER"`.
    pub role: Option<String>,
}
