use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A single notification line written to the JSONL stream consumed by Claude.
///
/// Every variant carries a `display` field containing the short one-line
/// message to surface to the operator. Variants are tagged with `kind` using
/// snake_case (e.g. `pr_comments`, `ci_failed`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NotificationLine {
    PrComments {
        owner: String,
        repo: String,
        pr: u64,
        branch: String,
        count: u32,
        digest_path: String,
        display: String,
    },
    PrLifecycle {
        owner: String,
        repo: String,
        pr: u64,
        branch: String,
        action: String,
        display: String,
    },
    CiFailed {
        owner: String,
        repo: String,
        branch: String,
        workflow: String,
        run_url: String,
        display: String,
    },
    FlushError {
        owner: String,
        repo: String,
        pr: u64,
        error: String,
        display: String,
    },
}

/// Append a single notification line to the JSONL file at `path`.
///
/// Creates the parent directory and the file itself if they do not exist.
/// On Unix, the file is created with mode `0o600` (owner read/write only).
/// Each call writes exactly one newline-terminated JSON object and fsyncs
/// the file so the line is durable before returning.
pub fn append_line(path: &Path, line: &NotificationLine) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).context("create notifications dir")?;
    }

    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    opts.mode(0o600);

    let mut f = opts.open(path).context("open notifications.jsonl")?;
    let s = serde_json::to_string(line).context("serialize notification line")?;
    writeln!(f, "{s}").context("write notifications line")?;
    // Best-effort durability: ignore fsync errors on platforms that don't support it.
    let _ = f.sync_all();
    Ok(())
}
