// Stub — replaced by Task 9.
use std::path::PathBuf;
use std::sync::Arc;

use super::types::{LogRetention, LogSystem};
use crate::config::LabConfig;
use crate::dispatch::error::ToolError;

pub fn clear_installed_log_system_for_test() {}

pub fn require_system() -> Result<Arc<LogSystem>, ToolError> {
    Err(ToolError::internal_message("log system stub (Task 9)"))
}

pub async fn bootstrap_running_log_system(
    _p: PathBuf,
    _r: LogRetention,
    _q: usize,
    _s: usize,
) -> anyhow::Result<Arc<LogSystem>> {
    unimplemented!("bootstrap_running_log_system stub (Task 9)")
}

pub async fn bootstrap_store_backed_log_system(
    _p: PathBuf,
    _r: LogRetention,
) -> anyhow::Result<Arc<LogSystem>> {
    unimplemented!("bootstrap_store_backed_log_system stub (Task 9)")
}

#[doc(hidden)]
pub async fn bootstrap_running_log_system_for_test(_q: usize) -> anyhow::Result<Arc<LogSystem>> {
    unimplemented!("bootstrap_running_log_system_for_test stub (Task 9)")
}

#[doc(hidden)]
pub fn bootstrap_log_system_for_test() -> anyhow::Result<Arc<LogSystem>> {
    unimplemented!("bootstrap_log_system_for_test stub (Task 9)")
}

pub fn resolve_store_path(_c: Option<&LabConfig>) -> PathBuf {
    PathBuf::new()
}

pub fn resolve_retention(_c: Option<&LabConfig>) -> LogRetention {
    LogRetention::default()
}

pub fn resolve_queue_capacity(_c: Option<&LabConfig>) -> usize {
    4096
}

pub fn resolve_subscriber_capacity(_c: Option<&LabConfig>) -> usize {
    1024
}
