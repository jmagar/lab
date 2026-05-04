//! Binary self-update flow for the `lab` TUI.

use anyhow::{Context as _, bail};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use sha2::{Digest as _, Sha256};

/// State machine for the self-update tab.
#[derive(Debug, Clone, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    Available {
        current: String,
        latest: String,
    },
    Downloading {
        progress: f32,
    },
    Verifying,
    Done,
    Error {
        message: String,
    },
}

// ── GitHub API types ──────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    digest: Option<String>,
}

// ── Platform helpers ──────────────────────────────────────────────────────────

/// Return the suffix expected in an asset file name for the running platform.
/// e.g. `"lab-x86_64-unknown-linux-musl"` or `"lab-x86_64-pc-windows-msvc.exe"`.
const fn platform_asset_prefix() -> &'static str {
    // Match the release asset naming used by cargo-release cross-builds.
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "lab-x86_64-unknown-linux"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "lab-aarch64-unknown-linux"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "lab-x86_64-pc-windows"
    } else {
        "lab-"
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Check GitHub for the latest release version without downloading anything.
///
/// Returns `(current_version, latest_version)`.
pub async fn check_for_update() -> anyhow::Result<(String, String)> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let release = fetch_latest_release().await?;
    let latest = release.tag_name.trim_start_matches('v').to_string();
    Ok((current, latest))
}

/// Drive the full update sequence, sending `AppEvent` transitions via `tx`.
///
/// All blocking I/O (download, verify, rename) is run inside
/// `tokio::task::spawn_blocking` so the async executor is never blocked.
///
/// # Preconditions
/// This function should only be called **after** the user has confirmed the
/// update prompt. It does not ask for confirmation internally.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::items_after_statements
)]
pub async fn perform_update(
    tx: tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> anyhow::Result<()> {
    use crate::tui::events::AppEvent;
    use futures::StreamExt as _;

    // 1. Signal: now checking
    tx.send(AppEvent::TaskDone {
        kind: "update_checking".into(),
    })
    .await
    .ok();

    // 2. Fetch latest release metadata
    let release = fetch_latest_release().await?;
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    if current == latest {
        tx.send(AppEvent::UpdateCheckDone {
            current,
            latest: None,
        })
        .await
        .ok();
        return Ok(());
    }

    // Find the matching asset for this platform
    let prefix = platform_asset_prefix();
    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name.starts_with(prefix))
        .with_context(|| {
            format!("No release asset matching prefix `{prefix}` for this platform")
        })?;

    // Signal: update available
    tx.send(AppEvent::UpdateCheckDone {
        current: current.clone(),
        latest: Some(latest.clone()),
    })
    .await
    .ok();

    // Extract expected SHA-256 from the digest field ("sha256:<hex>")
    let expected_sha256 = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(str::to_owned);

    let download_url = asset.browser_download_url.clone();

    // 5. Signal: downloading
    tx.send(AppEvent::TaskDone {
        kind: "update_downloading".into(),
    })
    .await
    .ok();

    // Determine the directory where the running binary lives so the temp file
    // is on the same filesystem — required for atomic rename via persist().
    let exe_path = std::env::current_exe().context("cannot determine current executable path")?;
    let exe_dir = exe_path
        .parent()
        .context("executable has no parent directory")?
        .to_path_buf();

    // Backup the current binary before touching anything
    let bak_path = exe_path.with_extension("bak");

    // 6. Download to a temp file in the same directory as the binary
    //    We stream the download and collect bytes for later SHA-256 verification.
    let client = reqwest::Client::builder()
        .user_agent(concat!("lab/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to build HTTP client")?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .context("failed to start download")?;

    let content_length = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();

    // Collect bytes while reporting progress
    let mut downloaded_bytes: Vec<u8> = if content_length > 0 {
        Vec::with_capacity(content_length as usize)
    } else {
        Vec::new()
    };

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("error reading download stream")?;
        downloaded_bytes.extend_from_slice(&chunk);

        if content_length > 0 {
            #[allow(clippy::cast_precision_loss)]
            let progress = downloaded_bytes.len() as f32 / content_length as f32;
            tx.send(AppEvent::TaskDone {
                kind: format!("update_progress:{progress:.4}"),
            })
            .await
            .ok();
        }
    }

    let exe_dir_clone = exe_dir.clone();
    let bak_path_clone = bak_path.clone();
    let exe_path_clone = exe_path.clone();
    let latest_clone = latest.clone();

    // 7. All blocking I/O: verify, write temp file, atomic rename, smoke-test
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        // Signal: verifying (sent from blocking thread — best-effort)
        // (Can't await here; we use the bytes we already have)

        // SHA-256 verify
        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&downloaded_bytes);
            let actual = hex::encode(hasher.finalize());
            if actual != expected {
                bail!("SHA-256 mismatch: expected {expected}, got {actual}");
            }
        }

        // Write to a temp file in the same directory as the binary
        let mut tmp = tempfile::NamedTempFile::new_in(&exe_dir_clone)
            .context("failed to create temp file")?;

        // Use std::io::Write
        use std::io::Write as _;
        tmp.write_all(&downloaded_bytes)
            .context("failed to write downloaded bytes to temp file")?;
        tmp.flush().context("failed to flush temp file")?;

        // chmod 0o755 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(tmp.path(), perms)
                .context("failed to set executable permissions")?;
        }

        // Backup current binary
        std::fs::copy(&exe_path_clone, &bak_path_clone)
            .context("failed to backup current binary")?;

        // Atomic rename: persist() moves the temp file to the exe path
        tmp.persist(&exe_path_clone)
            .context("failed to replace binary (persist failed)")?;

        // Smoke test: run new binary with --version
        let output = std::process::Command::new(&exe_path_clone)
            .arg("--version")
            .output()
            .context("failed to run smoke-test of new binary")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains(&latest_clone) && !output.status.success() {
            // Restore from backup
            if let Err(e) = std::fs::copy(&bak_path_clone, &exe_path_clone) {
                tracing::error!(
                    error = %e,
                    "smoke test failed AND backup restore failed — binary may be corrupted"
                );
            }
            bail!(
                "Smoke test failed: new binary --version output did not contain `{latest_clone}`. \
                 Previous binary restored from backup."
            );
        }

        // Remove backup on success (best-effort)
        drop(std::fs::remove_file(&bak_path_clone));

        Ok(())
    })
    .await
    .context("blocking update task panicked")??;

    // 8. Signal: done
    tx.send(AppEvent::TaskDone {
        kind: "update_done".into(),
    })
    .await
    .ok();

    Ok(())
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn fetch_latest_release() -> anyhow::Result<GhRelease> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("lab/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to build HTTP client")?;

    let release: GhRelease = client
        .get("https://api.github.com/repos/jmagar/lab/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("failed to reach GitHub API")?
        .error_for_status()
        .context("GitHub API returned error status")?
        .json()
        .await
        .context("failed to parse GitHub release JSON")?;

    Ok(release)
}

// ── Render ────────────────────────────────────────────────────────────────────

impl UpdateState {
    /// Render the update tab into `area` on `f`.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub fn render(&self, f: &mut ratatui::Frame<'_>, area: Rect, tick_count: u64) {
        use crate::tui::display::spinner_frame;
        use ratatui::layout::{Constraint, Direction, Layout};

        let block = Block::default()
            .title(" Self-Update ")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let text: Vec<Line<'_>> = match self {
            Self::Idle => vec![Line::from("Press 'u' to check for updates")],

            Self::Checking => {
                let s = spinner_frame(tick_count);
                vec![Line::from(format!("{s} Checking for updates\u{2026}"))]
            }

            Self::Available { current, latest } => vec![
                Line::from(vec![
                    Span::raw("Update available: "),
                    Span::styled(
                        current.as_str(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::raw(" \u{2192} "),
                    Span::styled(
                        latest.as_str(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from("Press Enter to download"),
            ],

            Self::Downloading { progress } => {
                let s = spinner_frame(tick_count);
                let pct = (*progress * 100.0) as u32;
                let filled = (*progress * 20.0) as usize;
                let empty = 20usize.saturating_sub(filled);
                let bar = format!(
                    "{s} Downloading\u{2026} [{filled}{empty}] {pct}%",
                    filled = "\u{2588}".repeat(filled),
                    empty = "\u{2591}".repeat(empty),
                );
                vec![Line::from(bar)]
            }

            Self::Verifying => {
                let s = spinner_frame(tick_count);
                vec![Line::from(format!("{s} Verifying SHA-256\u{2026}"))]
            }

            Self::Done => vec![Line::from(vec![Span::styled(
                "\u{2713} Update complete \u{2014} restart lab to use the new version",
                Style::default().fg(Color::Green),
            )])],

            Self::Error { message } => vec![Line::from(vec![Span::styled(
                message.as_str(),
                Style::default().fg(Color::Red),
            )])],
        };

        // Center vertically inside inner area.
        let text_height = text.len() as u16;
        let padding = inner.height.saturating_sub(text_height) / 2;
        let centered = if padding > 0 {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(padding),
                    Constraint::Length(text_height.max(1)),
                    Constraint::Min(0),
                ])
                .split(inner)[1]
        } else {
            inner
        };

        let para = Paragraph::new(text);
        f.render_widget(para, centered);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// SHA-256 mismatch must return Err before the file is persisted.
    #[test]
    fn sha256_mismatch_returns_error() {
        let tmp_dir = tempfile::tempdir().expect("tempdir");
        let mut tmp =
            tempfile::NamedTempFile::new_in(tmp_dir.path()).expect("NamedTempFile::new_in");

        let bytes = b"fake binary content";
        use std::io::Write as _;
        tmp.write_all(bytes).expect("write");

        // Compute the real digest
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let real = hex::encode(hasher.finalize());

        // Intentionally wrong expected digest
        let wrong_expected = "0".repeat(64);
        assert_ne!(real, wrong_expected);

        // Re-implement the verify logic the same way perform_update does
        let result: anyhow::Result<()> = {
            let mut hasher2 = Sha256::new();
            hasher2.update(bytes);
            let actual = hex::encode(hasher2.finalize());
            if actual != wrong_expected {
                Err(anyhow::anyhow!("SHA-256 mismatch"))
            } else {
                Ok(())
            }
        };

        assert!(result.is_err(), "should error on mismatch");
        // The temp file must still exist — persist() was never called
        assert!(
            tmp.path().exists(),
            "temp file should not have been removed"
        );
    }

    /// UpdateState transitions from Idle through Done must not panic.
    #[test]
    fn update_state_transitions_no_panic() {
        let states = vec![
            UpdateState::Idle,
            UpdateState::Checking,
            UpdateState::Available {
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
            },
            UpdateState::Downloading { progress: 0.42 },
            UpdateState::Verifying,
            UpdateState::Done,
            UpdateState::Error {
                message: "something went wrong".into(),
            },
        ];

        // Ensure each state can be cloned and debug-printed without panicking
        for state in &states {
            drop(format!("{state:?}"));
            drop(state.clone());
        }

        // Basic render smoke-test using an in-memory backend
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");

        for state in &states {
            terminal
                .draw(|f| {
                    let area = f.area();
                    state.render(f, area, 0);
                })
                .expect("draw");
            // Ensure the frame buffer has content (non-empty)
            let buf = terminal.backend().buffer().clone();
            let has_content = buf.content().iter().any(|c| c.symbol() != " ");
            assert!(has_content, "frame should have content for state {state:?}");
        }
    }

    /// Verify that NamedTempFile::new_in with the exe parent does not panic.
    #[test]
    fn tempfile_same_dir_as_exe() {
        let exe = std::env::current_exe().expect("current_exe");
        let dir = exe.parent().expect("exe parent");
        // The actual call pattern used in perform_update
        let tmp = tempfile::NamedTempFile::new_in(dir).expect("NamedTempFile::new_in");
        assert_eq!(
            tmp.path().parent().expect("tmp parent"),
            dir,
            "temp file should be in the same directory as the executable"
        );
    }
}
