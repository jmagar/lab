//! Prowlarr request/response types.
//!
//! TODO: add indexer, search result, and history types from the Prowlarr API spec.

use serde::{Deserialize, Serialize};

/// A Prowlarr indexer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    pub id: u64,
    pub name: String,
    pub protocol: String,
    pub enable: bool,
    pub priority: i32,
}
