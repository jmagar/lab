//! Flush pipeline: fetch PR comments since the last watermark, render a
//! digest, atomically write it to disk with `0o600`, and append a notification
//! line to JSONL. The watermark lives in-memory so edits-to-old-comments are
//! conservatively re-delivered on restart (never missed).

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use tracing::{error, info};

use crate::debounce::PrKey;
use crate::github::{Comment, GithubClient};
use crate::jsonl::{NotificationLine, append_line};
use crate::render::{is_safe_path_component, render_digest, safe_output_path};

/// Flushes pending comments for a PR. Shared across worker tasks via `Arc`.
pub struct Flusher {
    gh: GithubClient,
    data_dir: PathBuf,
    jsonl_path: PathBuf,
    // In-memory ISO8601 `updated_at` watermark per PR. Kept outside the
    // async boundary; std::sync::Mutex is correct because we never hold the
    // guard across an `.await`.
    watermarks: Arc<Mutex<HashMap<PrKey, String>>>,
}

impl Flusher {
    pub fn new(gh: GithubClient, data_dir: PathBuf, jsonl_path: PathBuf) -> Self {
        Self {
            gh,
            data_dir,
            jsonl_path,
            watermarks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Test/introspection helper: return the current watermark for a PR.
    pub fn watermark(&self, key: &PrKey) -> Option<String> {
        self.watermarks.lock().unwrap().get(key).cloned()
    }

    /// Flush pending comments for `key`. Swallows errors after logging them
    /// and appending a `flush_error` JSONL line — the caller (debouncer) does
    /// not need to surface the failure.
    pub async fn flush_pr(&self, key: PrKey, batch_count: u32) {
        match self.do_flush(&key, batch_count).await {
            Ok(()) => {}
            Err(e) => {
                error!(
                    target: "gh_webhook::flush",
                    owner = %key.owner, repo = %key.repo, pr = key.pr,
                    error = %e, "flush failed"
                );
                // Cap the error text so the JSONL line stays comfortably under
                // PIPE_BUF (4096 on Linux) and a pathological error message
                // can't poison the monitor stream.
                let mut error_text = format!("{e:#}");
                if error_text.len() > 500 {
                    error_text.truncate(500);
                    error_text.push('…');
                }
                let display = format!(
                    "[ERR] flush failed for {}/{} #{}: {}",
                    key.owner, key.repo, key.pr, error_text
                );
                let line = NotificationLine::FlushError {
                    owner: key.owner.clone(),
                    repo: key.repo.clone(),
                    pr: key.pr,
                    error: error_text,
                    display,
                };
                if let Err(append_err) = append_line(&self.jsonl_path, &line) {
                    error!(
                        target: "gh_webhook::flush",
                        error = %append_err, "failed to append flush_error line"
                    );
                }
            }
        }
    }

    async fn do_flush(&self, key: &PrKey, batch_count: u32) -> Result<()> {
        // Validate each component individually so we can safely compose a flat
        // filename like `owner-repo-pr.md` without traversal risk.
        if !is_safe_path_component(&key.owner) {
            anyhow::bail!("unsafe owner: {:?}", key.owner);
        }
        if !is_safe_path_component(&key.repo) {
            anyhow::bail!("unsafe repo: {:?}", key.repo);
        }
        let filename = format!("{}-{}-{}.md", key.owner, key.repo, key.pr);
        let digest_path = safe_output_path(&self.data_dir, &filename)?;

        let since = self.watermarks.lock().unwrap().get(key).cloned();
        let comments = self
            .gh
            .list_pr_comments(&key.owner, &key.repo, key.pr, since.as_deref())
            .await
            .context("list pr comments")?;

        let body = render_digest(&comments);
        let header = format!(
            "# {}/{}#{} — {} new comment(s)\n\n",
            key.owner,
            key.repo,
            key.pr,
            comments.len()
        );
        let rendered = format!("{header}{body}");

        if let Some(parent) = digest_path.parent() {
            std::fs::create_dir_all(parent).context("create data dir")?;
        }
        write_private(&digest_path, rendered.as_bytes())?;

        // Update the watermark to the max `updated_at` returned. GitHub's
        // `since=` filter is `updated_at`-based, so this gets edits-of-older-
        // comments next time at the cost of conservative re-delivery.
        if let Some(latest) = comments.iter().map(|c: &Comment| c.updated_at.clone()).max()
            && !latest.is_empty()
        {
            self.watermarks
                .lock()
                .unwrap()
                .insert(key.clone(), latest);
        }

        let digest_path_str = digest_path.to_string_lossy().into_owned();
        let fetched = comments.len() as u32;
        let display = format!(
            "[{batch}] NEW {fetched} comment(s) for {}/{} #{} — view at {}",
            key.owner,
            key.repo,
            key.pr,
            digest_path_str,
            batch = batch_count,
        );
        let line = NotificationLine::PrComments {
            owner: key.owner.clone(),
            repo: key.repo.clone(),
            pr: key.pr,
            // Branch is not known here — debouncer keys on (owner, repo, pr)
            // only. Pass empty string per the Task-10 adaptation note.
            branch: String::new(),
            count: fetched,
            digest_path: digest_path_str,
            display,
        };
        append_line(&self.jsonl_path, &line).context("append pr_comments line")?;

        info!(
            target: "gh_webhook::flush",
            owner = %key.owner, repo = %key.repo, pr = key.pr,
            count = fetched, batch = batch_count, "flushed"
        );
        Ok(())
    }
}

/// Atomic write: write to `<path>.tmp` with `0o600`, fsync, rename over the
/// target. Prevents a zero-byte `latest.md` or partial file after a crash.
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut opts = OpenOptions::new();
        opts.create(true).truncate(true).write(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let mut f = opts
            .open(&tmp)
            .with_context(|| format!("open {}", tmp.display()))?;
        f.write_all(bytes)
            .with_context(|| format!("write {}", tmp.display()))?;
        let _ = f.sync_all();
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}
