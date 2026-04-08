//! Radarr-specific filesystem types. The generic browse shapes
//! (`FilesystemEntry`, `FilesystemListing`, `FilesystemEntryType`) live in
//! [`crate::servarr::types::filesystem`]; manual-import carries a `MovieId`
//! reference and therefore stays here.

use serde::{Deserialize, Serialize};

pub use crate::servarr::types::filesystem::{
    FilesystemEntry, FilesystemEntryType, FilesystemListing,
};

use super::movie::MovieId;

/// One item available for manual import into radarr.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualImportItem {
    /// Import-item id.
    pub id: i64,
    /// Absolute path of the file on disk.
    pub path: String,
    /// Path relative to the import root.
    #[serde(default)]
    pub relative_path: Option<String>,
    /// File name.
    pub name: String,
    /// Size in bytes.
    pub size: i64,
    /// Movie the file was matched to, if any.
    #[serde(default)]
    pub movie_id: Option<MovieId>,
    /// Detected quality (dynamic shape — see Radarr OpenAPI spec).
    #[serde(default)]
    pub quality: serde_json::Value,
    /// Reasons the import was rejected, if any.
    #[serde(default)]
    pub rejections: serde_json::Value,
}
