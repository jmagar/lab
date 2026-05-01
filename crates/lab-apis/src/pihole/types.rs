//! Pi-hole response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generic JSON wrapper for Pi-hole API responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiholeResponse {
    pub value: Value,
}

/// Bounded query-log response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryLogResponse {
    pub value: Value,
    pub offset: u32,
    pub limit: u32,
}

/// Pi-hole v6 domain rule type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DomainType {
    ExactAllow,
    ExactDeny,
    RegexAllow,
    RegexDeny,
}

impl DomainType {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::ExactAllow => 0,
            Self::ExactDeny => 1,
            Self::RegexAllow => 2,
            Self::RegexDeny => 3,
        }
    }
}
