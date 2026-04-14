//! System, health, logs, updates, backups, tasks, disk space.
//!
//! Covers `/api/v3/system/*`, `/api/v3/health`, `/api/v3/ping`,
//! `/api/v3/log`, `/api/v3/update`, `/api/v3/diskspace`.
//!
//! Also expected to grow — when it does, nest as
//! `client/system/{status,health,log,update,backup,disk}.rs`.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::system::{DiskSpace, HealthCheck, LogFile, SystemStatus, UpdateInfo};

impl RadarrClient {
    /// Fetch system status — version, build time, runtime info.
    ///
    /// Maps to `GET /api/v3/system/status`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn system_status(&self) -> Result<SystemStatus, RadarrError> {
        self.http
            .get_json("/api/v3/system/status")
            .await
            .map_err(RadarrError::from)
    }

    /// List health-check warnings.
    ///
    /// Maps to `GET /api/v3/health`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn health_checks(&self) -> Result<Vec<HealthCheck>, RadarrError> {
        self.http
            .get_json("/api/v3/health")
            .await
            .map_err(RadarrError::from)
    }

    /// List recent log files.
    ///
    /// Maps to `GET /api/v3/log/file`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn log_files(&self) -> Result<Vec<LogFile>, RadarrError> {
        self.http
            .get_json("/api/v3/log/file")
            .await
            .map_err(RadarrError::from)
    }

    /// List available updates.
    ///
    /// Maps to `GET /api/v3/update`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn updates(&self) -> Result<Vec<UpdateInfo>, RadarrError> {
        self.http
            .get_json("/api/v3/update")
            .await
            .map_err(RadarrError::from)
    }

    /// Report free/used space for every known root folder.
    ///
    /// Maps to `GET /api/v3/diskspace`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn disk_space(&self) -> Result<Vec<DiskSpace>, RadarrError> {
        self.http
            .get_json("/api/v3/diskspace")
            .await
            .map_err(RadarrError::from)
    }

    /// Trigger a Radarr restart.
    ///
    /// Maps to `POST /api/v3/system/restart`. Destructive — the server will
    /// become briefly unavailable after this call returns.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn system_restart(&self) -> Result<(), RadarrError> {
        self.http
            .post_void("/api/v3/system/restart", &serde_json::Value::Null)
            .await
            .map_err(RadarrError::from)
    }

    /// List available backup files.
    ///
    /// Maps to `GET /api/v3/system/backup`. Returns metadata about all
    /// stored backups (name, path, type, size, creation time).
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn system_backup(&self) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json("/api/v3/system/backup")
            .await
            .map_err(RadarrError::from)
    }

    /// List all scheduled tasks (background jobs).
    ///
    /// Maps to `GET /api/v3/system/task`. Returns the task list showing each
    /// job's name, interval, last/next execution time, and current status.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn system_tasks(&self) -> Result<serde_json::Value, RadarrError> {
        self.http
            .get_json("/api/v3/system/task")
            .await
            .map_err(RadarrError::from)
    }
}
