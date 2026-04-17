// Stub — replaced by Task 5.
use super::types::{LogEvent, LogStreamReceiver, StreamSubscription};

pub struct StreamHub;

impl StreamHub {
    pub fn new(_capacity: usize) -> Self {
        Self
    }
    pub fn subscribe(&self, _s: StreamSubscription) -> LogStreamReceiver {
        unimplemented!("StreamHub stub (Task 5)")
    }
    pub fn publish(&self, _e: LogEvent) {}
}
