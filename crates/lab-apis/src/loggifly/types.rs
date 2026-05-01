//! LoggiFly DTOs.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ContractStatus {
    pub status: &'static str,
    pub reason: &'static str,
    pub safe_v1_actions: &'static [&'static str],
    pub deferred: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub enabled: bool,
    pub status: &'static str,
    pub heartbeat_path: String,
    pub heartbeat_interval_secs: u64,
    pub max_age_secs: u64,
    pub age_secs: Option<u64>,
    pub modified_unix_secs: Option<u64>,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSummary {
    pub config_root: Option<String>,
    pub config_path: Option<String>,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub modified_unix_secs: Option<u64>,
    pub containers_section: bool,
    pub notifications_section: bool,
    pub global_keywords_section: bool,
    pub raw_config_returned: bool,
}
