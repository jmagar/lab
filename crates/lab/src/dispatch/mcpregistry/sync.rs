//! Shared sync-guard state for the mcpregistry background supervisor and MCP dispatch.
//!
//! Both the hourly background supervisor (`cli/serve.rs`) and the on-demand MCP
//! `sync` action (`dispatch/mcpregistry/dispatch.rs`) must go through
//! `perform_sync` so that `SYNC_IN_PROGRESS` and `LAST_SYNC_AT` are visible
//! to both callers. Without this, a background tick and an MCP call can run
//! concurrently, producing interleaved SQLite write transactions.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use lab_apis::mcpregistry::McpRegistryClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::mcpregistry::store::RegistryStore;

/// Guards against concurrent syncs. `true` while a sync is in progress.
pub static SYNC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Tracks when the last successful sync completed.
pub static LAST_SYNC_AT: OnceLock<std::sync::Mutex<Option<std::time::Instant>>> = OnceLock::new();

/// RAII guard: resets `SYNC_IN_PROGRESS` on drop, even on panic.
pub struct SyncGuard;

impl Drop for SyncGuard {
    fn drop(&mut self) {
        SYNC_IN_PROGRESS.store(false, Ordering::Release);
    }
}

/// Minimum interval between syncs (enforced for on-demand calls only).
const MIN_SYNC_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

/// Attempt a sync, enforcing the concurrent-sync and rate-limit guards.
///
/// - `rate_limit`: when `true`, rejects calls within `MIN_SYNC_INTERVAL` of
///   the last successful sync. The background supervisor passes `false` since
///   its interval is already controlled by the `tokio::time::interval` timer.
/// - Returns the count of rows synced on success.
pub async fn perform_sync(
    store: &RegistryStore,
    client: &McpRegistryClient,
    rate_limit: bool,
) -> Result<usize, ToolError> {
    if SYNC_IN_PROGRESS
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return Err(ToolError::Sdk {
            sdk_kind: "rate_limited".to_string(),
            message: "sync already in progress".to_string(),
        });
    }
    let _guard = SyncGuard;

    if rate_limit {
        let last = LAST_SYNC_AT.get_or_init(|| std::sync::Mutex::new(None));
        let guard = last.lock().unwrap();
        if let Some(t) = *guard {
            if t.elapsed() < MIN_SYNC_INTERVAL {
                let remaining = MIN_SYNC_INTERVAL.saturating_sub(t.elapsed()).as_secs();
                return Err(ToolError::Sdk {
                    sdk_kind: "rate_limited".to_string(),
                    message: format!("sync rate-limited; next allowed in {remaining}s"),
                });
            }
        }
    }

    let count = store.sync_from_upstream(client).await.map_err(|e| {
        ToolError::internal_message(format!("sync failed: {e}"))
    })?;

    *LAST_SYNC_AT
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap() = Some(std::time::Instant::now());

    Ok(count)
}
