//! Prowlarr request/response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Prowlarr indexer configuration (minimal fields for display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexer {
    /// Prowlarr indexer ID.
    pub id: i64,
    pub name: String,
    pub protocol: String,
    pub enable: bool,
    pub priority: i32,
}

/// Request body for testing a single indexer.
///
/// Prowlarr requires sending the full indexer resource object to
/// `POST /api/v1/indexer/test`. We pass it through as a raw `Value`
/// to avoid duplicating the full schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerTestRequest(pub Value);

/// History query parameters.
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_key: Option<String>,
    pub sort_dir: Option<String>,
    pub indexer_id: Option<i64>,
}

impl HistoryQuery {
    /// Convert to query string pairs for `get_json_query`.
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(p) = self.page {
            pairs.push(("page".to_string(), p.to_string()));
        }
        if let Some(ps) = self.page_size {
            pairs.push(("pageSize".to_string(), ps.to_string()));
        }
        if let Some(ref sk) = self.sort_key {
            pairs.push(("sortKey".to_string(), sk.clone()));
        }
        if let Some(ref sd) = self.sort_dir {
            pairs.push(("sortDir".to_string(), sd.clone()));
        }
        if let Some(iid) = self.indexer_id {
            pairs.push(("indexerId".to_string(), iid.to_string()));
        }
        pairs
    }
}
