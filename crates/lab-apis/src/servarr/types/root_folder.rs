//! Root folder types.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a root-folder id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RootFolderId(pub i64);

/// A root folder an *arr service stores media under.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootFolder {
    pub id: RootFolderId,
    pub path: String,
    pub accessible: bool,
    pub free_space: i64,
    pub total_space: i64,
    #[serde(default)]
    pub unmapped_folders: serde_json::Value,
}
