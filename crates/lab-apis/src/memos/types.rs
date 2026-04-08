//! Memos request/response types.
//!
//! TODO: add memo, tag, and resource types from the Memos API spec.

use serde::{Deserialize, Serialize};

/// A Memos memo entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    pub id: u64,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub visibility: String,
}
