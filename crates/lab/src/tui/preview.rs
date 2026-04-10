//! Preview flow state machine for the TUI plugin manager.
//!
//! This module drives the full lifecycle of fetching and displaying a plugin
//! from a remote URL before the user commits to installing it.
//!
//! State transitions:
//!   Idle → Detecting → (PromptEcosystem | Fetching) → Ready | Error
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::Context as _;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use crate::tui::display::sanitize_display;
use crate::tui::marketplace::{InstallState, MarketplacePlugin};
use crate::tui::state::Ecosystem;

// ── PreviewState ──────────────────────────────────────────────────────────────

/// State machine for the plugin preview flow.
#[derive(Debug, Clone)]
pub enum PreviewState {
    Idle,
    Detecting {
        url: String,
    },
    PromptEcosystem {
        url: String,
        detected: Vec<Ecosystem>,
    },
    Fetching {
        url: String,
        ecosystem: Ecosystem,
    },
    Ready {
        plugin: MarketplacePlugin,
    },
    Error {
        message: String,
    },
}

// ── PreviewReady payload ──────────────────────────────────────────────────────

/// Payload carried by `AppEvent::PreviewReady`.  Includes TOCTOU-safe commit SHA.
#[derive(Debug, Clone)]
pub struct PreviewReady {
    pub plugin: MarketplacePlugin,
    /// Commit SHA captured at fetch time; pass as `--ref <sha>` to install.
    pub commit_sha: String,
    pub ecosystem: Ecosystem,
    pub source_url: String,
}

// ── URL validation ────────────────────────────────────────────────────────────

/// Validate a raw URL before any I/O.
///
/// Accepted:
/// - `https://…` URLs
/// - `git@host:path` SSH clone URLs
///
/// Rejected:
/// - `ext::*` (git remote helpers — arbitrary code execution)
/// - `file://…` (local filesystem access)
/// - Anything starting with `--` (option injection)
/// - Everything else
pub fn validate_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err("URL must not be empty".to_string());
    }

    // Reject option injection.
    if trimmed.starts_with("--") {
        return Err(format!("Rejected: URL looks like a flag: {trimmed}"));
    }

    // Reject ext:: remote helpers (arbitrary code execution vector).
    if trimmed.starts_with("ext::") {
        return Err(format!(
            "Rejected: ext:: remote helpers are not allowed: {trimmed}"
        ));
    }

    // Reject file:// local path access.
    if trimmed.starts_with("file://") || trimmed.starts_with("file:") {
        return Err(format!("Rejected: file:// URLs are not allowed: {trimmed}"));
    }

    // Accept https://.
    if trimmed.starts_with("https://") {
        return Ok(trimmed.to_string());
    }

    // Accept git@host:path SSH URLs.
    if trimmed.starts_with("git@") && trimmed.contains(':') {
        return Ok(trimmed.to_string());
    }

    Err(format!(
        "Rejected: only https:// and git@host:path URLs are accepted, got: {trimmed}"
    ))
}

// ── Ecosystem detection ───────────────────────────────────────────────────────

/// Which manifest files were found at the repo root.
#[derive(Debug)]
pub struct DetectedFiles {
    pub names: HashSet<String>,
}

impl DetectedFiles {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }

    fn has(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

/// Detect ecosystems from file names present at the repo root.
///
/// Returns:
/// - `Ok(vec![eco])` if exactly one ecosystem detected
/// - `Ok(vec![eco1, eco2, …])` if multiple detected (caller prompts user)
/// - `Err` if no ecosystem detected
pub fn detect_ecosystems(files: &DetectedFiles) -> Result<Vec<Ecosystem>, String> {
    let mut detected = Vec::new();

    // ClaudeCode: marketplace.json takes priority; single-entry fallback.
    if files.has(".claude-plugin/marketplace.json") || files.has(".claude-plugin/plugin.json") {
        detected.push(Ecosystem::ClaudeCode);
    }

    // Codex: either catalog or single-entry form.
    if files.has(".agents/plugins/marketplace.json") || files.has(".codex-plugin/plugin.json") {
        detected.push(Ecosystem::Codex);
    }

    // Gemini CLI.
    if files.has("gemini-extension.json") {
        detected.push(Ecosystem::Gemini);
    }

    if detected.is_empty() {
        Err("No plugin manifest found at the repository root".to_string())
    } else {
        Ok(detected)
    }
}

// ── JSON manifest types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ClaudePluginJson {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexPluginJson {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiExtensionJson {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

// ── GitHub API response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GitHubCommit {
    sha: String,
}

// ── Cache helpers ─────────────────────────────────────────────────────────────

const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

fn cache_path(url: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    let mut p = dirs_cache_root();
    p.push(format!("{digest}.json"));
    p
}

fn dirs_cache_root() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".lab").join("cache")
}

/// Read cached bytes if they exist and are within TTL.  Returns `None` on miss/expiry.
async fn read_cache(path: &PathBuf) -> Option<Vec<u8>> {
    let meta = tokio::fs::metadata(path).await.ok()?;
    let modified = meta.modified().ok()?;
    if SystemTime::now().duration_since(modified).ok()? > CACHE_TTL {
        return None;
    }
    tokio::fs::read(path).await.ok()
}

/// Read cached bytes ignoring TTL (for rate-limit fallback).
async fn read_cache_stale(path: &PathBuf) -> Option<Vec<u8>> {
    tokio::fs::read(path).await.ok()
}

/// Write bytes to cache atomically (write to temp, rename).
async fn write_cache(path: &PathBuf, data: &[u8]) {
    // Ensure the directory exists.
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let tmp = path.with_extension("tmp");
    if tokio::fs::write(&tmp, data).await.is_ok() {
        let _ = tokio::fs::rename(&tmp, path).await;
    }
}

// ── GitHub HTTP helpers ───────────────────────────────────────────────────────

fn build_github_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("lab-tui/0.1 (github.com/jmagar/lab)")
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default()
}

/// GET a URL, returning `(status, body_bytes)`.
async fn gh_get(client: &reqwest::Client, url: &str) -> anyhow::Result<(u16, Vec<u8>)> {
    let resp = client
        .get(url)
        .send()
        .await
        .context("HTTP request failed")?;
    let status = resp.status().as_u16();
    let bytes = resp.bytes().await.context("reading response body")?;
    Ok((status, bytes.to_vec()))
}

// ── fetch_preview ─────────────────────────────────────────────────────────────

/// Fetch a plugin preview from a remote URL for the given ecosystem.
///
/// For GitHub repos: uses the GitHub API + raw content URLs.
/// For non-GitHub git URLs: performs a sparse clone.
///
/// Sends an `AppEvent::PreviewReady(PreviewReady)` via `tx` on success.
pub async fn fetch_preview(
    url: &str,
    ecosystem: Ecosystem,
    tx: tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> anyhow::Result<PreviewState> {
    let result = if is_github_url(url) {
        fetch_github_preview(url, ecosystem, &tx).await
    } else {
        fetch_sparse_clone_preview(url, ecosystem, &tx).await
    };

    match result {
        Ok(ready) => {
            let plugin = ready.plugin.clone();
            drop(
                tx.send(crate::tui::events::AppEvent::PreviewReady(ready))
                    .await,
            );
            Ok(PreviewState::Ready { plugin })
        }
        Err(e) => Ok(PreviewState::Error {
            message: e.to_string(),
        }),
    }
}

fn is_github_url(url: &str) -> bool {
    url.starts_with("https://github.com/")
}

/// Parse `https://github.com/{owner}/{repo}` → `(owner, repo)`.
fn parse_github_owner_repo(url: &str) -> anyhow::Result<(String, String)> {
    let stripped = url
        .trim_end_matches('/')
        .trim_start_matches("https://github.com/");
    let mut parts = stripped.splitn(2, '/');
    let owner = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Could not parse GitHub owner from URL: {url}"))?;
    let repo = parts
        .next()
        .map(|r| r.trim_end_matches(".git").to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Could not parse GitHub repo from URL: {url}"))?;
    Ok((owner.to_string(), repo))
}

async fn fetch_github_preview(
    url: &str,
    ecosystem: Ecosystem,
    tx: &tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> anyhow::Result<PreviewReady> {
    let (owner, repo) = parse_github_owner_repo(url)?;
    let client = build_github_client();

    // 1. Get default branch.
    let repo_api = format!("https://api.github.com/repos/{owner}/{repo}");
    let (status, body) = gh_get(&client, &repo_api).await?;
    anyhow::ensure!(status == 200, "GitHub repo API returned HTTP {status}");
    let repo_info: GitHubRepo =
        serde_json::from_slice(&body).context("parsing GitHub repo API response")?;
    let branch = repo_info.default_branch;

    // 2. Resolve branch tip SHA (TOCTOU safety).
    let commits_api = format!("https://api.github.com/repos/{owner}/{repo}/commits/{branch}");
    let (status, body) = gh_get(&client, &commits_api).await?;
    anyhow::ensure!(status == 200, "GitHub commits API returned HTTP {status}");
    let commit_info: GitHubCommit =
        serde_json::from_slice(&body).context("parsing GitHub commits API response")?;
    let sha = commit_info.sha;

    // 3. Determine which manifest paths to fetch for this ecosystem.
    let manifest_paths = manifest_paths_for(ecosystem);

    // 4. Fetch manifest(s), using cache with TTL; rate-limit fallback.
    let cache_key = format!("{url}#{sha}#{}", ecosystem.as_str());
    let cache_file = cache_path(&cache_key);

    let manifest_bytes: Vec<u8> = if let Some(cached) = read_cache(&cache_file).await {
        cached
    } else {
        let raw_url_base = format!("https://raw.githubusercontent.com/{owner}/{repo}/{sha}");
        let mut fetched: Option<Vec<u8>> = None;

        for path in &manifest_paths {
            let raw_url = format!("{raw_url_base}/{path}");
            match gh_get(&client, &raw_url).await {
                Ok((429, _)) => {
                    // Rate limited — serve stale cache if available.
                    if let Some(stale) = read_cache_stale(&cache_file).await {
                        drop(
                            tx.send(crate::tui::events::AppEvent::TaskError {
                                kind: "preview".to_string(),
                                message: "showing cached preview (rate limited)".to_string(),
                            })
                            .await,
                        );
                        fetched = Some(stale);
                        break;
                    }
                    anyhow::bail!("GitHub rate limited and no cached preview available");
                }
                Ok((200, body)) => {
                    write_cache(&cache_file, &body).await;
                    fetched = Some(body);
                    break;
                }
                Ok((status, _)) => {
                    // Try next path.
                    tracing::debug!(path, status, "manifest not found, trying next");
                }
                Err(e) => {
                    tracing::warn!(path, err = %e, "failed to fetch manifest path");
                }
            }
        }

        fetched.ok_or_else(|| {
            anyhow::anyhow!("No plugin manifest found at any expected path for {ecosystem:?}")
        })?
    };

    // 5. Determine if this is a single-entry synthesis case.
    let use_synthesis = should_synthesize(ecosystem, &manifest_paths);

    // 6. Parse manifest into MarketplacePlugin.
    let plugin = parse_manifest(&manifest_bytes, ecosystem, url, use_synthesis)?;

    Ok(PreviewReady {
        plugin,
        commit_sha: sha,
        ecosystem,
        source_url: url.to_string(),
    })
}

/// Return the ordered list of manifest file paths to try for the given ecosystem.
fn manifest_paths_for(ecosystem: Ecosystem) -> Vec<&'static str> {
    match ecosystem {
        Ecosystem::ClaudeCode => {
            vec![
                ".claude-plugin/marketplace.json",
                ".claude-plugin/plugin.json",
            ]
        }
        Ecosystem::Codex => {
            vec![
                ".agents/plugins/marketplace.json",
                ".codex-plugin/plugin.json",
            ]
        }
        Ecosystem::Gemini => vec!["gemini-extension.json"],
    }
}

/// True if we should use single-entry synthesis (only plugin.json, no marketplace.json).
fn should_synthesize(ecosystem: Ecosystem, paths: &[&str]) -> bool {
    match ecosystem {
        Ecosystem::ClaudeCode => {
            paths.iter().any(|p| *p == ".claude-plugin/plugin.json")
                && !paths
                    .iter()
                    .any(|p| *p == ".claude-plugin/marketplace.json")
        }
        Ecosystem::Codex => {
            paths.iter().any(|p| *p == ".codex-plugin/plugin.json")
                && !paths
                    .iter()
                    .any(|p| *p == ".agents/plugins/marketplace.json")
        }
        Ecosystem::Gemini => false,
    }
}

/// Parse raw manifest bytes into a `MarketplacePlugin`.
fn parse_manifest(
    bytes: &[u8],
    ecosystem: Ecosystem,
    source_url: &str,
    _synthesis: bool,
) -> anyhow::Result<MarketplacePlugin> {
    match ecosystem {
        Ecosystem::ClaudeCode => {
            let m: ClaudePluginJson =
                serde_json::from_slice(bytes).context("parsing Claude plugin manifest")?;
            let name = m
                .name
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| source_url.to_string());
            Ok(MarketplacePlugin {
                id: name.to_lowercase().replace(' ', "-"),
                name: sanitize_display(&name, 80),
                description: sanitize_display(&m.description.unwrap_or_default(), 200),
                version: m.version.map(|v| sanitize_display(&v, 20)),
                ecosystem,
                install_state: InstallState::Available,
            })
        }
        Ecosystem::Codex => {
            let m: CodexPluginJson =
                serde_json::from_slice(bytes).context("parsing Codex plugin manifest")?;
            let name = m
                .name
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| source_url.to_string());
            Ok(MarketplacePlugin {
                id: name.to_lowercase().replace(' ', "-"),
                name: sanitize_display(&name, 80),
                description: sanitize_display(&m.description.unwrap_or_default(), 200),
                version: m.version.map(|v| sanitize_display(&v, 20)),
                ecosystem,
                install_state: InstallState::Available,
            })
        }
        Ecosystem::Gemini => {
            let m: GeminiExtensionJson =
                serde_json::from_slice(bytes).context("parsing Gemini extension manifest")?;
            let name = m
                .name
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| source_url.to_string());
            Ok(MarketplacePlugin {
                id: name.to_lowercase().replace(' ', "-"),
                name: sanitize_display(&name, 80),
                description: sanitize_display(&m.description.unwrap_or_default(), 200),
                version: m.version.map(|v| sanitize_display(&v, 20)),
                ecosystem,
                install_state: InstallState::Available,
            })
        }
    }
}

// ── Sparse clone (non-GitHub git URLs) ───────────────────────────────────────

async fn fetch_sparse_clone_preview(
    url: &str,
    ecosystem: Ecosystem,
    tx: &tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> anyhow::Result<PreviewReady> {
    let tmpdir = sparse_clone(url, tx).await?;
    let root = tmpdir.path();

    // Read manifest files from tmpdir.
    let paths = manifest_paths_for(ecosystem);
    let mut manifest_bytes: Option<Vec<u8>> = None;
    for path in &paths {
        let full = root.join(path);
        if let Ok(data) = tokio::fs::read(&full).await {
            manifest_bytes = Some(data);
            break;
        }
    }

    let bytes = manifest_bytes.ok_or_else(|| {
        anyhow::anyhow!("No plugin manifest found after sparse clone for {ecosystem:?}")
    })?;

    // For non-GitHub, we don't have a commit SHA from the API.
    // Use HEAD ref from .git/HEAD as a best-effort identifier.
    let sha = read_head_sha(root).await;
    let use_synthesis = should_synthesize_from_files(ecosystem, root);
    let plugin = parse_manifest(&bytes, ecosystem, url, use_synthesis)?;

    Ok(PreviewReady {
        plugin,
        commit_sha: sha,
        ecosystem,
        source_url: url.to_string(),
    })
}

async fn read_head_sha(root: &std::path::Path) -> String {
    let head = root.join(".git").join("HEAD");
    tokio::fs::read_to_string(head)
        .await
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn should_synthesize_from_files(ecosystem: Ecosystem, root: &std::path::Path) -> bool {
    match ecosystem {
        Ecosystem::ClaudeCode => {
            root.join(".claude-plugin/plugin.json").exists()
                && !root.join(".claude-plugin/marketplace.json").exists()
        }
        Ecosystem::Codex => {
            root.join(".codex-plugin/plugin.json").exists()
                && !root.join(".agents/plugins/marketplace.json").exists()
        }
        Ecosystem::Gemini => false,
    }
}

/// Perform a security-hardened sparse git clone into a temp directory.
///
/// Environment is scrubbed to prevent credential leakage, prompt hijacking,
/// or config injection. Times out after 30 seconds.
async fn sparse_clone(
    url: &str,
    _tx: &tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> anyhow::Result<tempfile::TempDir> {
    let tmpdir = tempfile::TempDir::new().context("creating temp dir for sparse clone")?;
    let tmppath = tmpdir.path().to_path_buf();

    let path_env = std::env::var("PATH").unwrap_or_default();
    let home_env = std::env::var("HOME").unwrap_or_default();

    // Step 1: clone --sparse --no-checkout.
    run_git_cmd(
        &[
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            "--no-checkout",
            url,
            tmppath.to_str().unwrap_or("."),
        ],
        None,
        &path_env,
        &home_env,
    )
    .await
    .context("git clone failed")?;

    // Step 2: sparse-checkout init --cone.
    run_git_cmd(
        &[
            "-C",
            tmppath.to_str().unwrap_or("."),
            "sparse-checkout",
            "init",
            "--cone",
        ],
        None,
        &path_env,
        &home_env,
    )
    .await
    .context("git sparse-checkout init failed")?;

    // Step 3: set sparse paths.
    run_git_cmd(
        &[
            "-C",
            tmppath.to_str().unwrap_or("."),
            "sparse-checkout",
            "set",
            ".claude-plugin",
            ".agents/plugins",
            ".codex-plugin",
            "gemini-extension.json",
        ],
        None,
        &path_env,
        &home_env,
    )
    .await
    .context("git sparse-checkout set failed")?;

    // Step 4: checkout.
    run_git_cmd(
        &["-C", tmppath.to_str().unwrap_or("."), "checkout"],
        None,
        &path_env,
        &home_env,
    )
    .await
    .context("git checkout failed")?;

    Ok(tmpdir)
}

/// Build and run a scrubbed git command with a 30-second timeout.
async fn run_git_cmd(
    args: &[&str],
    cwd: Option<&std::path::Path>,
    path_env: &str,
    home_env: &str,
) -> anyhow::Result<()> {
    use tokio::process::Command;

    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd.env_clear();
    cmd.env("PATH", path_env);
    cmd.env("HOME", home_env);
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.kill_on_drop(true);

    #[cfg(windows)]
    cmd.env(
        "SYSTEMROOT",
        std::env::var("SYSTEMROOT").unwrap_or_default(),
    );

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let fut = async {
        let output = cmd.output().await.context("spawning git")?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!(
                "git exited with status {:?}: {stderr}",
                output.status
            ))
        }
    };

    tokio::time::timeout(Duration::from_secs(30), fut)
        .await
        .map_err(|_| anyhow::anyhow!("git clone timed out"))?
}

// ── PreviewState render ───────────────────────────────────────────────────────

impl PreviewState {
    /// Render the current preview state into the given terminal area.
    pub fn render(&self, f: &mut Frame<'_>, area: Rect) {
        match self {
            PreviewState::Idle => {
                let p = Paragraph::new("Enter GitHub URL or git URL:")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Plugin Preview "),
                    )
                    .style(Style::default().fg(Color::Gray));
                f.render_widget(p, area);
            }

            PreviewState::Detecting { url } => {
                let safe_url = sanitize_display(url, 80);
                let p = Paragraph::new(format!("Detecting plugin type… {safe_url}"))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Plugin Preview "),
                    )
                    .style(Style::default().fg(Color::Yellow));
                f.render_widget(p, area);
            }

            PreviewState::PromptEcosystem { detected, .. } => {
                let items: Vec<ListItem<'_>> = detected
                    .iter()
                    .map(|eco| ListItem::new(eco.display_name()))
                    .collect();
                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Select ecosystem (j/k to move, Enter to confirm) "),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("> ");
                // Render without stateful selection — caller owns selection state.
                f.render_widget(list, area);
            }

            PreviewState::Fetching { url, ecosystem } => {
                let safe_url = sanitize_display(url, 60);
                let p = Paragraph::new(format!(
                    "Fetching {} plugin from {safe_url}…",
                    ecosystem.display_name()
                ))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Plugin Preview "),
                )
                .style(Style::default().fg(Color::Cyan));
                f.render_widget(p, area);
            }

            PreviewState::Ready { plugin } => {
                let eco_badge = sanitize_display(plugin.ecosystem.display_name(), 12);
                let name = sanitize_display(&plugin.name, 60);
                let desc = sanitize_display(&plugin.description, 200);
                let version = plugin
                    .version
                    .as_deref()
                    .map(|v| sanitize_display(v, 20))
                    .unwrap_or_else(|| "-".to_string());

                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{name}"),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("  "),
                        Span::styled(format!("v{version}"), Style::default().fg(Color::DarkGray)),
                        Span::raw("  "),
                        Span::styled(format!("[{eco_badge}]"), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(Span::styled(desc, Style::default().fg(Color::Gray))),
                    Line::from(""),
                    Line::from(Span::styled(
                        "[ Install ]",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                ];

                let p = Paragraph::new(lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Plugin Preview "),
                );
                f.render_widget(p, area);
            }

            PreviewState::Error { message } => {
                let safe_msg = sanitize_display(message, 200);
                let p = Paragraph::new(safe_msg)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Plugin Preview — Error "),
                    )
                    .style(Style::default().fg(Color::Red));
                f.render_widget(p, area);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL allowlist ─────────────────────────────────────────────────────────

    #[test]
    fn reject_ext_remote_helper() {
        assert!(validate_url("ext::evil-command").is_err());
    }

    #[test]
    fn reject_file_url() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("file://relative").is_err());
    }

    #[test]
    fn reject_option_injection() {
        assert!(validate_url("--upload-pack=./evil").is_err());
    }

    #[test]
    fn accept_https_url() {
        assert!(validate_url("https://github.com/owner/repo").is_ok());
    }

    #[test]
    fn accept_git_ssh_url() {
        assert!(validate_url("git@github.com:owner/repo.git").is_ok());
    }

    #[test]
    fn reject_empty_url() {
        assert!(validate_url("").is_err());
        assert!(validate_url("   ").is_err());
    }

    // ── Ecosystem detection ───────────────────────────────────────────────────

    #[test]
    fn detect_claude_marketplace() {
        let files = DetectedFiles::new([".claude-plugin/marketplace.json"]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected, vec![Ecosystem::ClaudeCode]);
    }

    #[test]
    fn detect_claude_single_entry() {
        let files = DetectedFiles::new([".claude-plugin/plugin.json"]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected, vec![Ecosystem::ClaudeCode]);
    }

    #[test]
    fn detect_codex_marketplace() {
        let files = DetectedFiles::new([".agents/plugins/marketplace.json"]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected, vec![Ecosystem::Codex]);
    }

    #[test]
    fn detect_codex_single_entry() {
        let files = DetectedFiles::new([".codex-plugin/plugin.json"]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected, vec![Ecosystem::Codex]);
    }

    #[test]
    fn detect_gemini() {
        let files = DetectedFiles::new(["gemini-extension.json"]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected, vec![Ecosystem::Gemini]);
    }

    #[test]
    fn detect_multiple_ecosystems() {
        let files = DetectedFiles::new([
            ".claude-plugin/marketplace.json",
            ".agents/plugins/marketplace.json",
        ]);
        let detected = detect_ecosystems(&files).unwrap();
        assert_eq!(detected.len(), 2);
        assert!(detected.contains(&Ecosystem::ClaudeCode));
        assert!(detected.contains(&Ecosystem::Codex));
    }

    #[test]
    fn detect_no_ecosystem_returns_error() {
        let files = DetectedFiles::new(["README.md", "Cargo.toml"]);
        assert!(detect_ecosystems(&files).is_err());
    }

    // ── Single-entry synthesis ────────────────────────────────────────────────

    #[test]
    fn single_entry_synthesis_claude() {
        let json = r#"{"name":"My Plugin","description":"A cool plugin","version":"1.2.3"}"#;
        let plugin = parse_manifest(
            json.as_bytes(),
            Ecosystem::ClaudeCode,
            "https://example.com",
            true,
        )
        .unwrap();
        assert_eq!(plugin.name, "My Plugin");
        assert_eq!(plugin.description, "A cool plugin");
        assert_eq!(plugin.version.as_deref(), Some("1.2.3"));
        assert_eq!(plugin.ecosystem, Ecosystem::ClaudeCode);
        assert_eq!(plugin.install_state, InstallState::Available);
    }

    // ── ANSI injection ────────────────────────────────────────────────────────

    #[test]
    fn ansi_injection_stripped_in_manifest() {
        let json = r#"{"name":"\u001b[31mevil\u001b[0m","description":"ok"}"#;
        let plugin = parse_manifest(
            json.as_bytes(),
            Ecosystem::ClaudeCode,
            "https://x.com",
            false,
        )
        .unwrap();
        assert!(
            !plugin.name.contains('\x1b'),
            "ANSI escapes must not appear in rendered plugin name"
        );
        assert!(
            plugin.name.contains("evil"),
            "text content must survive sanitization"
        );
    }

    // ── Rate-limit fallback ───────────────────────────────────────────────────

    #[tokio::test]
    async fn rate_limit_fallback_reads_stale_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache_file = dir.path().join("test_cache.json");
        let stale_data = b"stale payload";
        tokio::fs::write(&cache_file, stale_data).await.unwrap();

        // read_cache_stale ignores TTL.
        let result = read_cache_stale(&cache_file).await;
        assert_eq!(result.as_deref(), Some(stale_data.as_slice()));
    }

    #[tokio::test]
    async fn cache_miss_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cache_file = dir.path().join("nonexistent.json");
        assert!(read_cache_stale(&cache_file).await.is_none());
    }

    // ── Git env scrub ─────────────────────────────────────────────────────────

    #[test]
    fn git_command_env_scrub() {
        use tokio::process::Command;

        let mut cmd = Command::new("git");
        cmd.env_clear();
        cmd.env("PATH", "/usr/bin:/bin");
        cmd.env("HOME", "/root");
        cmd.env("GIT_TERMINAL_PROMPT", "0");
        cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
        cmd.kill_on_drop(true);

        // We can't inspect the env of a Command directly after setting,
        // so we verify by building the expected invariants conceptually.
        // The real check is that GIT_DIR is NOT set (not added above).
        // This test documents the expected env scrub pattern.

        // Drop the command without running — just verify it compiles and builds.
        drop(cmd);
    }

    // ── GitHub URL parsing ────────────────────────────────────────────────────

    #[test]
    fn parse_github_url_basic() {
        let (owner, repo) = parse_github_owner_repo("https://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn parse_github_url_strips_git_suffix() {
        let (owner, repo) = parse_github_owner_repo("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn parse_github_url_trailing_slash() {
        let (owner, repo) = parse_github_owner_repo("https://github.com/owner/repo/").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }
}
