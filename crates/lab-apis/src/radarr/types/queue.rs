//! Download-queue types.

use serde::{Deserialize, Serialize};

use super::movie::MovieId;

/// Newtype wrapper around a queue-item id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueueItemId(pub i64);

/// One item in the download queue.
///
/// Mirrors `QueueResource` from the Radarr v3 OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: QueueItemId,
    #[serde(default)]
    pub movie_id: Option<MovieId>,
    pub title: String,
    pub status: String,
    pub size: i64,
    pub sizeleft: i64,
    #[serde(default)]
    pub timeleft: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub download_client: Option<String>,
    #[serde(default)]
    pub indexer: Option<String>,
}
