//! Audit report types.

use serde::{Deserialize, Serialize};

/// Result of one audit check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckResult {
    Pass,
    Fail(String),
    Skip(String),
}

/// All checks for one service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceReport {
    pub service: String,
    pub checks: Vec<(String, CheckResult)>,
}

/// Audit report for multiple services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    pub services: Vec<ServiceReport>,
}

impl AuditReport {
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.services
            .iter()
            .any(|service| service.checks.iter().any(|(_, result)| matches!(result, CheckResult::Fail(_))))
    }
}
