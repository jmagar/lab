//! SABnzbd request/response types.
//!
//! TODO: add queue, history item, and status types from the SABnzbd API spec.

use serde::{Deserialize, Serialize};

/// A `SABnzbd` queue item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub nzo_id: String,
    pub filename: String,
    pub status: String,
    pub percentage: String,
    pub mb: String,
    pub mbleft: String,
    pub timeleft: String,
}
