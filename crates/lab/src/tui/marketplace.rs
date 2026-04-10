//! Marketplace browser tab — TUI plugin discovery for Claude Code, Codex, Gemini CLI.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexSet;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::tui::display::sanitize_display;
use crate::tui::state::Ecosystem;

// ── PluginId newtype ──────────────────────────────────────────────────────────

/// Normalized plugin identifier — lowercase, `-`-separated, trimmed.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PluginId(String);

impl PluginId {
    /// Normalize: lowercase, replace `-`, `_`, and spaces with `-`, trim.
    pub fn new(raw: &str) -> Self {
        let normalized = raw
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if matches!(c, '-' | '_' | ' ') { '-' } else { c })
            .collect::<String>();
        // Collapse consecutive dashes.
        let mut result = String::with_capacity(normalized.len());
        let mut last_dash = false;
        for c in normalized.chars() {
            if c == '-' {
                if !last_dash {
                    result.push(c);
                }
                last_dash = true;
            } else {
                result.push(c);
                last_dash = false;
            }
        }
        // Trim leading/trailing dashes.
        let result = result.trim_matches('-').to_string();
        Self(result)
    }

    /// Return the underlying normalized string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── InstallState ──────────────────────────────────────────────────────────────

/// Installation state of a marketplace plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    Available,
    Installed,
    UpdateAvailable,
}

impl InstallState {
    fn badge(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Installed => "installed",
            Self::UpdateAvailable => "update",
        }
    }

    fn style(self) -> Style {
        match self {
            Self::Available => Style::default().fg(Color::DarkGray),
            Self::Installed => Style::default().fg(Color::Green),
            Self::UpdateAvailable => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

// ── MarketplacePlugin ─────────────────────────────────────────────────────────

/// A plugin entry from a marketplace catalog.
#[derive(Debug, Clone)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub install_state: InstallState,
}

// ── CliPresence ───────────────────────────────────────────────────────────────

/// Whether the Claude Code and Gemini CLI binaries are on PATH.
pub struct CliPresence {
    pub claude: bool,
    pub gemini: bool,
}

impl CliPresence {
    /// Probe PATH for `claude` and `gemini` binaries.
    ///
    /// Uses `tokio::task::spawn_blocking` so the blocking `which` subprocess
    /// does not stall the async executor.
    pub async fn detect() -> Self {
        let (claude, gemini) = tokio::join!(
            tokio::task::spawn_blocking(|| {
                std::process::Command::new("which")
                    .arg("claude")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }),
            tokio::task::spawn_blocking(|| {
                std::process::Command::new("which")
                    .arg("gemini")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }),
        );
        Self {
            claude: claude.unwrap_or(false),
            gemini: gemini.unwrap_or(false),
        }
    }
}

// ── MarketplaceState ──────────────────────────────────────────────────────────

/// State for the Plugins/Marketplace tab.
#[derive(Debug)]
pub struct MarketplaceState {
    pub plugins: Vec<MarketplacePlugin>,
    pub selected: usize,
    pub loading: bool,
    pub preview: Option<crate::tui::preview::PreviewState>,
    /// Filter view to a single ecosystem, or show all when `None`.
    pub filter_ecosystem: Option<Ecosystem>,
}

impl Default for MarketplaceState {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            selected: 0,
            loading: true,
            preview: None,
            filter_ecosystem: None,
        }
    }
}

impl MarketplaceState {
    /// Render the marketplace list into the given area.
    pub fn render(&self, f: &mut Frame<'_>, area: Rect, tick_count: u64) {
        use crate::tui::display::spinner_frame;
        use ratatui::widgets::Paragraph;

        if self.loading {
            let spinner = spinner_frame(tick_count);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Plugins — Marketplace ");
            let para = Paragraph::new(format!("{spinner} Loading marketplace catalogs\u{2026}"))
                .block(block);
            f.render_widget(para, area);
            return;
        }

        let visible: Vec<&MarketplacePlugin> = match self.filter_ecosystem {
            None => self.plugins.iter().collect(),
            Some(eco) => self.plugins.iter().filter(|p| p.ecosystem == eco).collect(),
        };

        if visible.is_empty() {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Plugins — Marketplace ");
            let para = Paragraph::new("Press i to add a marketplace or extension").block(block);
            f.render_widget(para, area);
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Plugins — Marketplace ");

        let items: Vec<ListItem<'_>> = visible
            .iter()
            .map(|p| {
                let name = sanitize_display(&p.name, 30);
                let desc = sanitize_display(&p.description, 40);
                let eco = p.ecosystem.display_name();
                let state_badge = p.install_state.badge();

                let eco_color = match p.ecosystem {
                    Ecosystem::ClaudeCode => Color::Cyan,
                    Ecosystem::Codex => Color::Yellow,
                    Ecosystem::Gemini => Color::Blue,
                };

                let line = Line::from(vec![
                    Span::styled(format!("{name:<30}"), Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(format!("{desc:<40}"), Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                    Span::styled(format!("{eco:<11}"), Style::default().fg(eco_color)),
                    Span::raw(" "),
                    Span::styled(format!("{state_badge:<9}"), p.install_state.style()),
                ]);
                ListItem::new(line)
            })
            .collect();

        let mut list_state = ListState::default();
        if !visible.is_empty() {
            list_state.select(Some(self.selected.min(visible.len().saturating_sub(1))));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut list_state);
    }
}

// ── JSON serde helpers (file I/O only) ────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ClaudeMarketplaceIndex {
    #[serde(rename = "installLocation")]
    install_location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudePluginManifest {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexMarketplaceEntry {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiExtensionManifest {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
}

// ── MarketplaceLoader ─────────────────────────────────────────────────────────

/// Data layer: loads marketplace catalogs from local files across all ecosystems.
pub struct MarketplaceLoader;

impl MarketplaceLoader {
    /// Load all marketplace sources across all three ecosystems concurrently.
    /// Uses `FuturesUnordered` + `Arc<Semaphore>` (limit 15).
    /// Returns a unified `Vec<MarketplacePlugin>` with install_state cross-referenced.
    pub async fn load_all(cli: &CliPresence) -> Vec<MarketplacePlugin> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let sem = Arc::new(Semaphore::new(15));

        // Build installed sets eagerly (sync, cheap).
        let claude_installed = Self::installed_claude();
        let codex_installed = Self::installed_codex();
        let gemini_installed = Self::installed_gemini();

        let mut futures: FuturesUnordered<tokio::task::JoinHandle<Vec<(MarketplacePlugin, bool)>>> =
            FuturesUnordered::new();

        // ── Claude Code sources ───────────────────────────────────────────────
        if cli.claude {
            let s = sem.clone();
            futures.push(tokio::spawn(async move {
                let _p = s.acquire_owned().await.ok();
                load_claude_plugins().await
            }));
        }

        // ── Codex sources ─────────────────────────────────────────────────────
        {
            let s = sem.clone();
            futures.push(tokio::spawn(async move {
                let _p = s.acquire_owned().await.ok();
                load_codex_plugins().await
            }));
        }

        // ── Gemini CLI sources ────────────────────────────────────────────────
        if cli.gemini {
            let s = sem.clone();
            futures.push(tokio::spawn(async move {
                let _p = s.acquire_owned().await.ok();
                load_gemini_plugins().await
            }));
        }

        // Collect all results.
        let mut raw: Vec<(MarketplacePlugin, bool)> = Vec::new();
        while let Some(res) = futures.next().await {
            if let Ok(batch) = res {
                raw.extend(batch);
            }
        }

        // Apply install-state cross-reference.
        raw.into_iter()
            .map(|(mut plugin, _)| {
                let pid = PluginId::new(&plugin.id);
                plugin.install_state = match plugin.ecosystem {
                    Ecosystem::ClaudeCode => {
                        if claude_installed.contains(&pid) {
                            InstallState::Installed
                        } else {
                            InstallState::Available
                        }
                    }
                    Ecosystem::Codex => {
                        if codex_installed.contains(&pid) {
                            InstallState::Installed
                        } else {
                            InstallState::Available
                        }
                    }
                    Ecosystem::Gemini => {
                        if gemini_installed.contains(&pid) {
                            InstallState::Installed
                        } else {
                            InstallState::Available
                        }
                    }
                };
                plugin
            })
            .collect()
    }

    // ── Installed-set builders ────────────────────────────────────────────────

    /// Claude Code: `~/.claude/plugins/installed_plugins.json` — JSON object, keys are plugin names.
    fn installed_claude() -> IndexSet<PluginId> {
        let path = home_dir().join(".claude/plugins/installed_plugins.json");
        let Ok(data) = std::fs::read_to_string(&path) else {
            return IndexSet::new();
        };
        let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&data) else {
            return IndexSet::new();
        };
        map.keys().map(|k| PluginId::new(k)).collect()
    }

    /// Codex: `~/.codex/config.toml` — TOML, section names are plugin ids.
    fn installed_codex() -> IndexSet<PluginId> {
        let path = home_dir().join(".codex/config.toml");
        let Ok(data) = std::fs::read_to_string(&path) else {
            return IndexSet::new();
        };
        let Ok(table) = toml::from_str::<toml::Value>(&data) else {
            return IndexSet::new();
        };
        if let Some(toml::Value::Table(t)) = Some(table) {
            t.keys().map(|k| PluginId::new(k)).collect()
        } else {
            IndexSet::new()
        }
    }

    /// Gemini: `~/.gemini/extensions/` directory presence — dirname = plugin id.
    fn installed_gemini() -> IndexSet<PluginId> {
        let dir = home_dir().join(".gemini/extensions");
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return IndexSet::new();
        };
        rd.filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .map(|name| PluginId::new(&name))
            .collect()
    }
}

// ── Source loaders ────────────────────────────────────────────────────────────

/// Return `(plugin, /* unused sentinel */ false)` pairs.
async fn load_claude_plugins() -> Vec<(MarketplacePlugin, bool)> {
    let index_path = home_dir().join(".claude/plugins/known_marketplaces.json");
    let Ok(data) = tokio::fs::read_to_string(&index_path).await else {
        return vec![];
    };
    let Ok(entries) = serde_json::from_str::<Vec<ClaudeMarketplaceIndex>>(&data) else {
        return vec![];
    };

    let mut plugins = Vec::new();
    for entry in entries {
        let Some(loc) = entry.install_location else {
            continue;
        };
        let manifest_path = PathBuf::from(&loc).join(".claude-plugin/marketplace.json");
        let Ok(mdata) = tokio::fs::read_to_string(&manifest_path).await else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<ClaudePluginManifest>(&mdata) else {
            continue;
        };
        let name = manifest.name.unwrap_or_else(|| loc.clone());
        let id = PluginId::new(&name).to_string();
        plugins.push((
            MarketplacePlugin {
                id,
                name,
                description: manifest.description.unwrap_or_default(),
                version: manifest.version,
                ecosystem: Ecosystem::ClaudeCode,
                install_state: InstallState::Available,
            },
            false,
        ));
    }
    plugins
}

async fn load_codex_plugins() -> Vec<(MarketplacePlugin, bool)> {
    let mut plugins = Vec::new();

    // Personal catalog.
    let personal = home_dir().join(".agents/plugins/marketplace.json");
    plugins.extend(load_codex_catalog(&personal).await);

    // Repo-local catalog (optional).
    if let Ok(cwd) = std::env::current_dir() {
        let repo_catalog = cwd.join(".agents/plugins/marketplace.json");
        plugins.extend(load_codex_catalog(&repo_catalog).await);
    }

    plugins
}

async fn load_codex_catalog(path: &PathBuf) -> Vec<(MarketplacePlugin, bool)> {
    let Ok(data) = tokio::fs::read_to_string(path).await else {
        return vec![];
    };
    let Ok(entries) = serde_json::from_str::<Vec<CodexMarketplaceEntry>>(&data) else {
        return vec![];
    };
    entries
        .into_iter()
        .filter_map(|e| {
            let name = e.name?;
            let id = PluginId::new(&name).to_string();
            Some((
                MarketplacePlugin {
                    id,
                    name,
                    description: e.description.unwrap_or_default(),
                    version: e.version,
                    ecosystem: Ecosystem::Codex,
                    install_state: InstallState::Available,
                },
                false,
            ))
        })
        .collect()
}

async fn load_gemini_plugins() -> Vec<(MarketplacePlugin, bool)> {
    let ext_dir = home_dir().join(".gemini/extensions");
    let Ok(mut rd) = tokio::fs::read_dir(&ext_dir).await else {
        return vec![];
    };

    let mut plugins = Vec::new();
    while let Ok(Some(entry)) = rd.next_entry().await {
        if !entry.path().is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("gemini-extension.json");
        let Ok(data) = tokio::fs::read_to_string(&manifest_path).await else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<GeminiExtensionManifest>(&data) else {
            continue;
        };
        let dir_name = entry.file_name().into_string().unwrap_or_default();
        let name = manifest.name.unwrap_or_else(|| dir_name.clone());
        let id = PluginId::new(&name).to_string();
        plugins.push((
            MarketplacePlugin {
                id,
                name,
                description: manifest.description.unwrap_or_default(),
                version: manifest.version,
                ecosystem: Ecosystem::Gemini,
                install_state: InstallState::Available,
            },
            false,
        ));
    }
    plugins
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a catalog with given names for given ecosystems.
    fn make_plugin(name: &str, ecosystem: Ecosystem, state: InstallState) -> MarketplacePlugin {
        MarketplacePlugin {
            id: PluginId::new(name).to_string(),
            name: name.to_string(),
            description: String::new(),
            version: None,
            ecosystem,
            install_state: state,
        }
    }

    #[test]
    fn install_state_cross_reference() {
        // Catalog: A (claude), B (codex), C (claude — not installed).
        let mut plugins = vec![
            make_plugin("plugin-a", Ecosystem::ClaudeCode, InstallState::Available),
            make_plugin("plugin-b", Ecosystem::Codex, InstallState::Available),
            make_plugin("plugin-c", Ecosystem::ClaudeCode, InstallState::Available),
        ];

        let mut claude_installed: IndexSet<PluginId> = IndexSet::new();
        claude_installed.insert(PluginId::new("plugin-a"));

        let mut codex_installed: IndexSet<PluginId> = IndexSet::new();
        codex_installed.insert(PluginId::new("plugin-b"));

        let gemini_installed: IndexSet<PluginId> = IndexSet::new();

        // Apply cross-reference (same logic as load_all).
        for plugin in &mut plugins {
            let pid = PluginId::new(&plugin.id);
            plugin.install_state = match plugin.ecosystem {
                Ecosystem::ClaudeCode => {
                    if claude_installed.contains(&pid) {
                        InstallState::Installed
                    } else {
                        InstallState::Available
                    }
                }
                Ecosystem::Codex => {
                    if codex_installed.contains(&pid) {
                        InstallState::Installed
                    } else {
                        InstallState::Available
                    }
                }
                Ecosystem::Gemini => {
                    if gemini_installed.contains(&pid) {
                        InstallState::Installed
                    } else {
                        InstallState::Available
                    }
                }
            };
        }

        assert_eq!(
            plugins[0].install_state,
            InstallState::Installed,
            "plugin-a should be installed"
        );
        assert_eq!(
            plugins[1].install_state,
            InstallState::Installed,
            "plugin-b should be installed"
        );
        assert_eq!(
            plugins[2].install_state,
            InstallState::Available,
            "plugin-c should be available"
        );
    }

    #[test]
    fn id_normalization_matches_across_formats() {
        // "My-Plugin" in catalog, "my_plugin" in installed set — should match.
        let catalog_id = PluginId::new("My-Plugin");
        let installed_id = PluginId::new("my_plugin");
        assert_eq!(
            catalog_id, installed_id,
            "normalized IDs must match across dash/underscore/case variants"
        );
    }

    #[test]
    fn ecosystem_enum_exhaustive() {
        // Instantiate ALL variants so the compiler catches missing arms.
        for eco in Ecosystem::ALL {
            let _s = eco.as_str();
            let _d = eco.display_name();
            match eco {
                Ecosystem::ClaudeCode => {}
                Ecosystem::Codex => {}
                Ecosystem::Gemini => {}
            }
        }
    }

    #[test]
    fn ansi_injection_stripped() {
        let ansi_name = "\x1b[31mEvil Plugin\x1b[0m";
        let plugin = make_plugin(ansi_name, Ecosystem::ClaudeCode, InstallState::Available);
        // The plugin name goes through sanitize_display before rendering.
        let safe = sanitize_display(&plugin.name, 200);
        assert!(
            !safe.contains('\x1b'),
            "ANSI escapes must be stripped from plugin names"
        );
        assert!(
            safe.contains("Evil Plugin"),
            "text content must survive sanitization"
        );
    }

    #[test]
    fn plugin_id_collapses_consecutive_separators() {
        let id = PluginId::new("my--plugin___name");
        assert_eq!(id.as_str(), "my-plugin-name");
    }

    #[test]
    fn plugin_id_trims_leading_trailing_dashes() {
        let id = PluginId::new("  -my-plugin-  ");
        assert_eq!(id.as_str(), "my-plugin");
    }
}
