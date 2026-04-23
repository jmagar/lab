//! No external HTTP client — `marketplace` is a local-only synthetic service.
//!
//! This file satisfies the required dispatch service layout contract
//! (every migrated service must have `client.rs`). All work is local
//! filesystem I/O plus optional `tokio::process::Command` shell-out to
//! `claude plugin install/uninstall`.

use std::path::PathBuf;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

#[cfg(test)]
static TEST_PLUGINS_ROOT_OVERRIDE: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
#[cfg(test)]
static TEST_PLUGINS_ROOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(super) fn plugins_root() -> Result<PathBuf, ToolError> {
    #[cfg(test)]
    if let Some(path) = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap().clone() {
        return Ok(path);
    }

    let home = env_non_empty("HOME").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "HOME env var not set".into(),
    })?;
    Ok(PathBuf::from(home).join(".claude").join("plugins"))
}

#[cfg(test)]
pub(super) fn with_test_plugins_root<T>(home: &std::path::Path, run: impl FnOnce() -> T) -> T {
    let _guard = TEST_PLUGINS_ROOT_LOCK.lock().unwrap();
    let plugins_root = home.join(".claude").join("plugins");
    let previous = {
        let mut slot = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap();
        std::mem::replace(&mut *slot, Some(plugins_root))
    };
    let result = run();
    let mut slot = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap();
    *slot = previous;
    result
}
