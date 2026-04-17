// Stub — replaced by Task 4.
use super::types::{LogQuery, LogRetention, LogSearchResult, LogStoreStats, LogTailRequest, LogTailResult};
use crate::dispatch::error::ToolError;

pub struct LogStore;

impl LogStore {
    pub async fn search(&self, _q: LogQuery) -> Result<LogSearchResult, ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
    pub async fn tail(&self, _r: LogTailRequest) -> Result<LogTailResult, ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
    pub async fn stats(&self) -> Result<LogStoreStats, ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
    pub async fn run_maintenance(&self) -> Result<(), ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
    pub async fn insert(&self, _e: &super::types::LogEvent) -> Result<(), ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
    pub async fn open(_p: std::path::PathBuf, _r: LogRetention) -> Result<Self, ToolError> {
        unimplemented!("LogStore stub (Task 4)")
    }
}

#[doc(hidden)]
pub async fn open_store_for_test(_r: LogRetention) -> Result<LogStore, ToolError> {
    unimplemented!("open_store_for_test stub (Task 4)")
}
