//! LoggiFly contract status.

use crate::core::Auth;

use super::error::LoggiflyError;
use super::types::ContractStatus;

#[derive(Clone, Default)]
pub struct LoggiflyClient {}

impl LoggiflyClient {
    pub fn new(_base_url: &str, _auth: Auth) -> Result<Self, LoggiflyError> {
        Ok(Self {})
    }

    pub fn contract_status(&self) -> ContractStatus {
        ContractStatus {
            status: "implementation_deferred",
            reason: "Public docs describe config files, Docker labels, notifications, and healthcheck files, but no stable safe HTTP API was found.",
            safe_v1_actions: &["contract.status"],
            deferred: &[
                "config.summary",
                "config.validate",
                "health.status",
                "docker.logs",
                "docker.labels",
                "notification.test",
                "action.trigger",
            ],
        }
    }

    pub async fn health(&self) -> Result<(), LoggiflyError> {
        Ok(())
    }
}
