//! System status, health, log, update, and disk-space types.

use serde::{Deserialize, Serialize};

/// System status — version, runtime, paths. Shape is identical across *arr
/// services; `app_name` disambiguates which product is responding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct SystemStatus {
    pub version: String,
    pub build_time: String,
    pub is_debug: bool,
    pub is_production: bool,
    pub is_admin: bool,
    pub is_user_interactive: bool,
    pub startup_path: String,
    pub app_data: String,
    pub os_name: String,
    #[serde(default)]
    pub os_version: Option<String>,
    pub is_linux: bool,
    pub is_osx: bool,
    pub is_windows: bool,
    pub is_docker: bool,
    pub mode: String,
    pub branch: String,
    pub runtime_version: String,
    pub runtime_name: String,
    pub app_name: String,
    pub instance_name: Option<String>,
    #[serde(default)]
    pub database_type: Option<String>,
    #[serde(default)]
    pub database_version: Option<String>,
}

/// Severity of a health check finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthSeverity {
    Ok,
    Notice,
    Warning,
    Error,
}

/// One health-check entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheck {
    pub source: String,
    #[serde(rename = "type")]
    pub severity: HealthSeverity,
    pub message: String,
    #[serde(default)]
    pub wiki_url: Option<String>,
}

/// A log file recorded by an *arr service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFile {
    pub id: i64,
    pub filename: String,
    pub last_write_time: String,
    #[serde(alias = "contentsUrl")]
    pub content_url: String,
    pub download_url: String,
}

/// One available update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub branch: String,
    pub release_date: String,
    #[serde(default)]
    pub file_name: Option<String>,
    pub installable: bool,
    pub latest: bool,
    #[serde(default)]
    pub changes: serde_json::Value,
}

/// Disk-space report for a mount point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    pub path: String,
    pub label: String,
    pub free_space: i64,
    pub total_space: i64,
}
