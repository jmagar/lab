//! History and blocklist types.

use serde::{Deserialize, Serialize};

use super::movie::MovieId;

/// Newtype wrapper around a history-record id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HistoryId(pub i64);

/// Newtype wrapper around a blocklist-entry id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlocklistId(pub i64);

/// What happened to a release — Radarr distinguishes grab, import,
/// upgrade, and failure events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HistoryEventType {
    Unknown,
    Grabbed,
    DownloadFolderImported,
    DownloadFailed,
    MovieFileDeleted,
    MovieFolderImported,
    MovieFileRenamed,
    DownloadIgnored,
}

/// A single history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: HistoryId,
    pub movie_id: MovieId,
    pub source_title: String,
    pub event_type: HistoryEventType,
    pub date: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// A page of history results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub page: u32,
    pub page_size: u32,
    pub total_records: u32,
    pub records: Vec<HistoryRecord>,
}

/// A release on the blocklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlocklistItem {
    pub id: BlocklistId,
    pub movie_id: MovieId,
    pub source_title: String,
    pub date: String,
    #[serde(default)]
    pub indexer: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}
