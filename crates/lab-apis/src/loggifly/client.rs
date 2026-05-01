//! LoggiFly local config and heartbeat inspection.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::Auth;

use super::error::LoggiflyError;
use super::types::{ConfigSummary, ContractStatus, HealthStatus};

const DEFAULT_HEARTBEAT_PATH: &str = "/dev/shm/loggifly-heartbeat";
const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 60;

#[derive(Clone)]
pub struct LoggiflyClient {
    config_root: Option<PathBuf>,
    heartbeat_path: PathBuf,
    heartbeat_interval_secs: u64,
}

impl Default for LoggiflyClient {
    fn default() -> Self {
        Self {
            config_root: None,
            heartbeat_path: PathBuf::from(DEFAULT_HEARTBEAT_PATH),
            heartbeat_interval_secs: DEFAULT_HEARTBEAT_INTERVAL_SECS,
        }
    }
}

impl LoggiflyClient {
    pub fn new(_base_url: &str, _auth: Auth) -> Result<Self, LoggiflyError> {
        Ok(Self::default())
    }

    pub fn with_local_paths(
        config_root: Option<PathBuf>,
        heartbeat_path: Option<PathBuf>,
        heartbeat_interval_secs: Option<u64>,
    ) -> Self {
        Self {
            config_root,
            heartbeat_path: heartbeat_path.unwrap_or_else(|| PathBuf::from(DEFAULT_HEARTBEAT_PATH)),
            heartbeat_interval_secs: heartbeat_interval_secs
                .unwrap_or(DEFAULT_HEARTBEAT_INTERVAL_SECS)
                .max(3),
        }
    }

    pub fn contract_status(&self) -> ContractStatus {
        ContractStatus {
            status: "local_contract_implemented",
            reason: "LoggiFly has no stable HTTP API. Lab implements the documented local heartbeat-file health contract and redacted config summary.",
            safe_v1_actions: &["contract.status", "health.status", "config.summary"],
            deferred: &[
                "config.validate",
                "docker.logs",
                "docker.labels",
                "notification.test",
                "action.trigger",
            ],
        }
    }

    pub async fn health_status(&self) -> Result<HealthStatus, LoggiflyError> {
        let max_age_secs = self.heartbeat_interval_secs + self.heartbeat_interval_secs / 2;
        let metadata = match tokio::fs::metadata(&self.heartbeat_path).await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(HealthStatus {
                    enabled: false,
                    status: "missing",
                    heartbeat_path: self.heartbeat_path.display().to_string(),
                    heartbeat_interval_secs: self.heartbeat_interval_secs,
                    max_age_secs,
                    age_secs: None,
                    modified_unix_secs: None,
                    reason: Some(
                        "heartbeat file missing; enable LoggiFly healthcheck with ENABLE_HEALTHCHECK=true",
                    ),
                });
            }
            Err(err) => return Err(LoggiflyError::Io(err.to_string())),
        };
        let modified = metadata
            .modified()
            .map_err(|err| LoggiflyError::Io(err.to_string()))?;
        let modified_unix_secs = unix_secs(modified)?;
        let now_secs = unix_secs(SystemTime::now())?;
        let age_secs = now_secs.saturating_sub(modified_unix_secs);
        let stale = age_secs > max_age_secs;
        Ok(HealthStatus {
            enabled: true,
            status: if stale { "stale" } else { "healthy" },
            heartbeat_path: self.heartbeat_path.display().to_string(),
            heartbeat_interval_secs: self.heartbeat_interval_secs,
            max_age_secs,
            age_secs: Some(age_secs),
            modified_unix_secs: Some(modified_unix_secs),
            reason: stale.then_some("heartbeat file is older than HEARTBEAT_INTERVAL * 1.5"),
        })
    }

    pub async fn config_summary(&self) -> Result<ConfigSummary, LoggiflyError> {
        let Some(root) = &self.config_root else {
            return Ok(ConfigSummary {
                config_root: None,
                config_path: None,
                exists: false,
                size_bytes: None,
                modified_unix_secs: None,
                containers_section: false,
                notifications_section: false,
                global_keywords_section: false,
                raw_config_returned: false,
            });
        };
        let path = root.join("config.yaml");
        let metadata = match tokio::fs::metadata(&path).await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ConfigSummary {
                    config_root: Some(root.display().to_string()),
                    config_path: Some(path.display().to_string()),
                    exists: false,
                    size_bytes: None,
                    modified_unix_secs: None,
                    containers_section: false,
                    notifications_section: false,
                    global_keywords_section: false,
                    raw_config_returned: false,
                });
            }
            Err(err) => return Err(LoggiflyError::Io(err.to_string())),
        };
        let modified_unix_secs = metadata
            .modified()
            .ok()
            .and_then(|time| unix_secs(time).ok());
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|err| LoggiflyError::Io(err.to_string()))?;
        Ok(ConfigSummary {
            config_root: Some(root.display().to_string()),
            config_path: Some(path.display().to_string()),
            exists: true,
            size_bytes: Some(metadata.len()),
            modified_unix_secs,
            containers_section: has_top_level_key(&text, "containers"),
            notifications_section: has_top_level_key(&text, "notifications"),
            global_keywords_section: has_top_level_key(&text, "global_keywords"),
            raw_config_returned: false,
        })
    }

    pub async fn health(&self) -> Result<(), LoggiflyError> {
        let status = self.health_status().await?;
        if status.status == "healthy" {
            Ok(())
        } else {
            Err(LoggiflyError::Io(format!(
                "LoggiFly health status is {}",
                status.status
            )))
        }
    }
}

fn has_top_level_key(text: &str, key: &str) -> bool {
    let needle = format!("{key}:");
    text.lines().any(|line| {
        let trimmed = line.trim_end();
        trimmed == needle || trimmed.starts_with(&format!("{needle} "))
    })
}

fn unix_secs(time: SystemTime) -> Result<u64, LoggiflyError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| LoggiflyError::Io(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_heartbeat_reports_missing() {
        let dir = tempfile::tempdir().unwrap();
        let client =
            LoggiflyClient::with_local_paths(None, Some(dir.path().join("missing")), Some(60));

        let status = client.health_status().await.unwrap();

        assert_eq!(status.status, "missing");
        assert!(!status.enabled);
    }

    #[tokio::test]
    async fn fresh_heartbeat_reports_healthy() {
        let dir = tempfile::tempdir().unwrap();
        let heartbeat = dir.path().join("heartbeat");
        tokio::fs::write(&heartbeat, b"ok").await.unwrap();
        let client = LoggiflyClient::with_local_paths(None, Some(heartbeat), Some(60));

        let status = client.health_status().await.unwrap();

        assert_eq!(status.status, "healthy");
        assert!(status.enabled);
    }

    #[tokio::test]
    async fn config_summary_detects_sections_without_returning_raw_config() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(
            dir.path().join("config.yaml"),
            "containers:\n  app:\n    keywords:\n      - error\nnotifications:\n  ntfy:\n    token: secret\nglobal_keywords:\n  keywords:\n    - failed\n",
        )
        .await
        .unwrap();
        let client = LoggiflyClient::with_local_paths(Some(dir.path().to_path_buf()), None, None);

        let summary = client.config_summary().await.unwrap();

        assert!(summary.exists);
        assert!(summary.containers_section);
        assert!(summary.notifications_section);
        assert!(summary.global_keywords_section);
        assert!(!summary.raw_config_returned);
    }

    #[test]
    fn max_uses_minimum_heartbeat_interval() {
        let client = LoggiflyClient::with_local_paths(None, None, Some(1));
        assert_eq!(client.heartbeat_interval_secs, 3);
    }

    #[test]
    fn has_top_level_key_ignores_nested_keys() {
        assert!(has_top_level_key("containers:\n", "containers"));
        assert!(!has_top_level_key("  containers:\n", "containers"));
        assert!(!has_top_level_key("containers_extra:\n", "containers"));
        assert!(!has_top_level_key("", "containers"));
    }
}
