//! Memos request/response types.
//!
//! Targets the **Memos v1 API** (resource-name style, RFC3339 timestamps).
//! TODO: add tag and resource upload types from the Memos API spec.

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
