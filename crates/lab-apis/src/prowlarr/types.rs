//! Prowlarr request/response types.
//!
//! TODO: add indexer, search result, and history types from the Prowlarr API spec.

use serde::{Deserialize, Serialize};

/// A Prowlarr indexer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    /// Prowlarr indexer ID (signed, consistent with other Servarr services).
    pub id: i64,
    pub name: String,
    pub protocol: String,
    pub enable: bool,
    pub priority: i32,
}
