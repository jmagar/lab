//! Beads response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Beads CLI contract metadata.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ContractStatus {
    pub status: &'static str,
    pub reason: &'static str,
    pub safe_v1_actions: &'static [&'static str],
    pub deferred: &'static [&'static str],
}

/// Generic JSON output from a read-only `bd --json` command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BdJson {
    pub command: String,
    pub value: Value,
}

/// Health/status result for the local Beads workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeadsHealth {
    pub bd_available: bool,
    pub status: &'static str,
    pub version: Option<String>,
    pub context: Option<Value>,
    pub summary: Option<Value>,
    pub message: Option<String>,
}
