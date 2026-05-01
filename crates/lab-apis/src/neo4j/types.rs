//! Neo4j request and response types.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Default maximum rows returned by a Cypher action.
pub const MAX_ROWS: usize = 500;

/// Generic bounded Cypher response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CypherResponse {
    pub rows: Vec<Map<String, Value>>,
    pub row_count: usize,
    pub truncated: bool,
}

/// Response for a successful write/discarding action.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WriteResponse {
    pub ok: bool,
    pub statements: usize,
}

/// Transaction statement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Statement {
    pub statement: String,
    #[serde(default)]
    pub parameters: Map<String, Value>,
}

/// Transaction execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxMode {
    R,
    W,
}
