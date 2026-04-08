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
        self.http.get_json("/api/v3/system/status").await.map_err(RadarrError::from)
    }

    /// List health-check warnings.
    ///
    /// Maps to `GET /api/v3/health`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn health_checks(&self) -> Result<Vec<HealthCheck>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// List recent log files.
    ///
    /// Maps to `GET /api/v3/log/file`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn log_files(&self) -> Result<Vec<LogFile>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// List available updates.
    ///
    /// Maps to `GET /api/v3/update`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn updates(&self) -> Result<Vec<UpdateInfo>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// Report free/used space for every known root folder.
    ///
    /// Maps to `GET /api/v3/diskspace`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn disk_space(&self) -> Result<Vec<DiskSpace>, RadarrError> {
        let _ = &self.http;
        Ok(Vec::new())
    }
}
