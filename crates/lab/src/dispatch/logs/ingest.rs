// Stub — replaced by Task 6.
use std::sync::Arc;

use super::store::LogStore;
use super::stream::StreamHub;
use super::types::RawLogEvent;
use crate::dispatch::error::ToolError;

pub struct IngestHandle;

impl IngestHandle {
    pub async fn submit(&self, _r: RawLogEvent) -> Result<(), ToolError> {
        unimplemented!("IngestHandle stub (Task 6)")
    }
    pub fn try_submit(&self, _r: RawLogEvent) -> Result<(), ToolError> {
        unimplemented!("IngestHandle stub (Task 6)")
    }
}

pub struct IngestCounters;

impl IngestCounters {
    pub fn new() -> Self {
        Self
    }
    pub fn dropped(&self) -> u64 {
        0
    }
}

impl Default for IngestCounters {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spawn_writer(
    _store: Arc<LogStore>,
    _hub: Arc<StreamHub>,
    _queue_capacity: usize,
) -> (IngestHandle, Arc<IngestCounters>) {
    unimplemented!("spawn_writer stub (Task 6)")
}

pub fn readonly_handle(_counters: Arc<IngestCounters>) -> IngestHandle {
    IngestHandle
}

/// Tracing layer that forwards events into the installed LogSystem. Stub until Task 6.
pub struct LogIngestLayer;

impl<S> tracing_subscriber::Layer<S> for LogIngestLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // No-op stub; real implementation in Task 6.
    }
}
