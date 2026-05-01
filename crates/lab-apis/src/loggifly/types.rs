//! LoggiFly DTOs.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ContractStatus {
    pub status: &'static str,
    pub reason: &'static str,
    pub safe_v1_actions: &'static [&'static str],
    pub deferred: &'static [&'static str],
}
