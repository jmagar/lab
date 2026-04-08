//! Filesystem browsing types.

use serde::{Deserialize, Serialize};

/// Whether a filesystem entry is a folder or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemEntryType {
    File,
    Folder,
    Drive,
}

/// One entry in a filesystem browse response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesystemEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: FilesystemEntryType,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub size: Option<i64>,
}

/// Response from `/api/v3/filesystem?path=...`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesystemListing {
    pub parent: Option<String>,
    pub directories: Vec<FilesystemEntry>,
    pub files: Vec<FilesystemEntry>,
}
