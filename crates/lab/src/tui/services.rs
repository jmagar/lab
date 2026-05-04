//! Service manager tab — displays compiled-in services, their env var status,
//! health dots, and allows toggling MCP wiring via `.mcp.json`.

use std::collections::HashMap;

use indexmap::IndexSet;
use std::path::PathBuf;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use secrecy::SecretString;

use crate::tui::display::sanitize_display;
use crate::tui::events::ServiceHealth;
use crate::tui::metadata::all_services;

/// Return the path to `~/.lab/.env`, falling back to `/root/.lab/.env` when
/// `HOME` is unset (containers, some CI, sandboxed contexts).
///
/// `PathBuf::from("~/.lab/.env")` does **not** expand the tilde — shells do.
/// The `/root` fallback is the conventional home directory in rootful containers.
pub fn lab_env_path() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from("/root/.lab/.env"),
        |h| PathBuf::from(h).join(".lab").join(".env"),
    )
}

/// Visual health indicator for a single service row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthDot {
    /// Reachable + auth ok.
    GreenOk,
    /// Reachable + auth fail.
    YellowAuth,
    /// Not reachable.
    RedUnreach,
    /// Env vars missing — cannot attempt connection.
    GreyNoEnv,
    /// Not in `--services` list (not enabled in MCP).
    DimNotEnabled,
}

impl HealthDot {
    const fn symbol(self) -> &'static str {
        match self {
            Self::GreenOk | Self::YellowAuth | Self::RedUnreach => "●",
            Self::GreyNoEnv => "○",
            Self::DimNotEnabled => "·",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::GreenOk => Color::Green,
            Self::YellowAuth => Color::Yellow,
            Self::RedUnreach => Color::Red,
            Self::GreyNoEnv | Self::DimNotEnabled => Color::DarkGray,
        }
    }
}

/// State for the Services tab.
pub struct LabServicesState {
    /// Index of currently selected service row.
    pub selected: usize,
    /// Health results keyed by service name.
    pub health: HashMap<String, ServiceHealth>,
    /// Whether secret env var values are revealed (`r` key toggles).
    pub reveal_secret: bool,
    /// Path to the `.mcp.json` file being managed.
    pub mcp_json_path: Option<PathBuf>,
    /// Set of service names currently listed in `.mcp.json` `--services`.
    pub enabled_services: IndexSet<String>,
    /// Cached env vars from `~/.lab/.env`. Loaded once at startup and after
    /// the user returns from the `$EDITOR` session. Never read in the render path.
    pub env_cache: HashMap<String, String>,
    /// Monotonically increasing counter bumped each time a health check is spawned.
    /// `HealthChecksDone` events carrying a stale generation are discarded.
    pub health_generation: u64,
    /// Ratatui list state for scroll tracking.
    list_state: ListState,
}

impl Default for LabServicesState {
    /// Returns an empty state immediately — no blocking I/O.
    ///
    /// Actual seeding (`.mcp.json` + `.env` cache) is deferred to a
    /// `spawn_blocking` task that posts [`AppEvent::ServicesSeeded`].
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            selected: 0,
            health: HashMap::new(),
            reveal_secret: false,
            mcp_json_path: None,
            enabled_services: IndexSet::new(),
            env_cache: HashMap::new(),
            health_generation: 0,
            list_state,
        }
    }
}

/// Perform the blocking startup I/O for the Services tab.
///
/// Reads `.mcp.json` to determine which services are enabled, and loads the
/// `.env` cache. Intended to be called via `tokio::task::spawn_blocking` so
/// that file I/O never blocks the tokio executor.
pub fn load_initial_state() -> (Option<PathBuf>, IndexSet<String>, HashMap<String, String>) {
    let path = default_mcp_json_path();
    let enabled = seed_enabled_services(path.as_deref());
    let env = load_env_vars();
    (path, enabled, env)
}

fn default_mcp_json_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".claude").join(".mcp.json"))
}

/// Seed `enabled_services` from the current `.mcp.json` content.
///
/// Called at startup so the initial state matches what's actually configured.
/// Non-fatal: if the file is missing or malformed, returns an empty set.
///
/// # Atomic rename contract
///
/// This function reads `.mcp.json` with [`std::fs::read_to_string`] without
/// taking any lock. It is safe only because **all write paths in `mcp_patch.rs`
/// use a write-to-temp-then-rename strategy** (`NamedTempFile::persist`), so
/// the kernel never exposes a partially-written file. If any write path in
/// `mcp_patch.rs` were to truncate-and-write instead of rename, a race window
/// would exist where this function sees an empty or partial file and silently
/// returns an empty set. **Do not add direct `fs::write` calls to `.mcp.json`.**
///
/// The format written by `mcp_patch.rs` stores each service as a separate
/// element in the `args` array immediately after `--services`:
/// ```json
/// { "mcpServers":{"labby": { "args": ["mcp", "--services", "radarr", "sonarr"] } } }
/// ```
pub fn seed_enabled_services(mcp_json_path: Option<&std::path::Path>) -> IndexSet<String> {
    let Some(path) = mcp_json_path else {
        return IndexSet::new();
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return IndexSet::new();
    };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(&content) else {
        return IndexSet::new();
    };
    let args = json
        .pointer("/mcpServers/lab/args")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut services = IndexSet::new();
    let mut collecting = false;
    for arg in &args {
        let s = arg.as_str().unwrap_or("");
        if s == "--services" {
            collecting = true;
        } else if collecting {
            if s.starts_with('-') {
                // Hit another flag — stop collecting services.
                collecting = false;
            } else {
                // The write format uses individual args, but handle comma-separated
                // values defensively in case of hand-edited files.
                for svc in s.split(',') {
                    let svc = svc.trim();
                    if !svc.is_empty() {
                        services.insert(svc.to_string());
                    }
                }
            }
        }
    }
    services
}

/// Read the current service env vars from `~/.lab/.env`, masking secrets.
fn load_env_vars() -> HashMap<String, String> {
    let env_path = lab_env_path();

    dotenvy::from_path_iter(&env_path)
        .ok()
        .map(|it| it.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

impl LabServicesState {
    /// Move selection down by one row.
    pub fn select_next(&mut self) {
        let count = all_services().len();
        if count == 0 {
            return;
        }
        self.selected = (self.selected + 1).min(count - 1);
        self.list_state.select(Some(self.selected));
    }

    /// Move selection up by one row.
    pub const fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    /// Toggle secret reveal mode.
    pub const fn toggle_reveal(&mut self) {
        self.reveal_secret = !self.reveal_secret;
    }

    /// Reload env vars from `~/.lab/.env` into the cache.
    /// Call after the user returns from an editor session.
    pub fn reload_env_cache(&mut self) {
        self.env_cache = load_env_vars();
    }

    /// Reload the enabled-services set from `.mcp.json`.
    /// Call after the user returns from an editor session that may have toggled services.
    pub fn reload_enabled_services(&mut self) {
        self.enabled_services = seed_enabled_services(self.mcp_json_path.as_deref());
    }

    /// Update health results from a completed health check pass.
    ///
    /// Replaces the map wholesale so stale entries for removed services are cleared.
    pub fn update_health(&mut self, results: Vec<ServiceHealth>) {
        self.health = results
            .into_iter()
            .map(|h| (h.service.clone(), h))
            .collect();
    }

    /// Toggle whether a service is enabled in `.mcp.json`.
    pub fn toggle_enabled(&mut self) -> anyhow::Result<()> {
        let plugins = all_services();
        let Some(plugin) = plugins.get(self.selected) else {
            return Ok(());
        };
        let name = plugin.name;
        let path = match &self.mcp_json_path {
            Some(p) => p.clone(),
            None => anyhow::bail!("no .mcp.json path configured"),
        };
        let currently_enabled = self.enabled_services.contains(name);
        crate::tui::mcp_patch::patch_mcp_json(&path, name, !currently_enabled)?;
        if currently_enabled {
            self.enabled_services.swap_remove(name);
        } else {
            self.enabled_services.insert(name.to_owned());
        }
        Ok(())
    }

    /// Open `~/.lab/.env` in `$EDITOR`.
    ///
    /// **Important:** the caller is responsible for restoring and re-initializing
    /// the terminal around this call, as the editor takes over the terminal.
    pub fn open_env_editor() -> anyhow::Result<()> {
        let env_path = lab_env_path();

        // Ensure parent directory and file exist.
        if let Some(parent) = env_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !env_path.exists() {
            std::fs::write(&env_path, "# Lab service configuration\n")?;
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        // Shell-split so `EDITOR="code --wait"` works correctly.
        let parts: Vec<&str> = editor.split_ascii_whitespace().collect();
        let program = parts.first().copied().unwrap_or("vi");
        let args = &parts[1..];
        std::process::Command::new(program)
            .args(args)
            .arg(&env_path)
            .status()
            .map_err(|e| anyhow::anyhow!("failed to launch editor '{editor}': {e}"))?;
        Ok(())
    }

    /// Compute the health dot for a service.
    fn health_dot(&self, name: &str) -> HealthDot {
        if !self.enabled_services.contains(name) {
            return HealthDot::DimNotEnabled;
        }
        match self.health.get(name) {
            None => HealthDot::GreyNoEnv,
            Some(h) if !h.reachable => HealthDot::RedUnreach,
            Some(h) if !h.auth_ok => HealthDot::YellowAuth,
            Some(_) => HealthDot::GreenOk,
        }
    }

    /// Render the full Services tab into `area`.
    pub fn render(&mut self, frame: &mut Frame<'_>, area: Rect, _tick_count: u64) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        self.render_service_list(frame, chunks[0]);
        self.render_detail_panel(frame, chunks[1]);

        // Reveal banner at the bottom of the right panel when active.
        if self.reveal_secret {
            let banner_area = Rect {
                y: chunks[1].y + chunks[1].height.saturating_sub(1),
                height: 1,
                ..chunks[1]
            };
            let banner = Paragraph::new("  ⚠  Secrets revealed — press 'r' to hide").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(banner, banner_area);
        }
    }

    fn render_service_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let plugins = all_services();
        let items: Vec<ListItem<'_>> = plugins
            .iter()
            .map(|p| {
                let dot = self.health_dot(p.name);
                let dot_span = Span::styled(
                    format!("{} ", dot.symbol()),
                    Style::default().fg(dot.color()),
                );
                let enabled_marker = if self.enabled_services.contains(p.name) {
                    Span::styled("[on] ", Style::default().fg(Color::Green))
                } else {
                    Span::styled("[--] ", Style::default().fg(Color::DarkGray))
                };
                let name_span = Span::raw(sanitize_display(p.name, 20));
                let cat_span = Span::styled(
                    format!(" ({})", p.category),
                    Style::default().fg(Color::DarkGray),
                );
                ListItem::new(Line::from(vec![
                    dot_span,
                    enabled_marker,
                    name_span,
                    cat_span,
                ]))
            })
            .collect();

        let block = Block::default()
            .title(" Services ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    #[allow(clippy::too_many_lines)]
    fn render_detail_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let plugins = all_services();
        let Some(plugin) = plugins.get(self.selected) else {
            return;
        };

        let env_vars = &self.env_cache;
        let mut lines: Vec<Line<'_>> = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled(
                sanitize_display(plugin.name, 30),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — "),
            Span::styled(
                sanitize_display(plugin.category, 20),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            sanitize_display(plugin.description, 80),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        // Health status
        if self.enabled_services.contains(plugin.name) {
            match self.health.get(plugin.name) {
                None => {
                    lines.push(Line::from(Span::styled(
                        "○ No health data — press F5 to refresh",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                Some(h) => {
                    let (symbol, color, msg) = if !h.reachable {
                        (
                            "● Unreachable",
                            Color::Red,
                            h.message.as_deref().unwrap_or(""),
                        )
                    } else if !h.auth_ok {
                        (
                            "● Auth failed",
                            Color::Yellow,
                            h.message.as_deref().unwrap_or(""),
                        )
                    } else {
                        ("● Healthy", Color::Green, "")
                    };
                    let mut health_line = vec![Span::styled(symbol, Style::default().fg(color))];
                    if !msg.is_empty() {
                        health_line.push(Span::raw(format!(": {}", sanitize_display(msg, 60))));
                    }
                    if let Some(ms) = h.latency_ms {
                        health_line.push(Span::styled(
                            format!(" ({ms}ms)"),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    lines.push(Line::from(health_line));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "· Not enabled in MCP — press Space to enable",
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::from(""));

        // Required env vars
        // Read env vars from lab_apis META constants via a helper.
        let required = plugin_required_env(plugin.name);
        let optional = plugin_optional_env(plugin.name);

        if !required.is_empty() {
            lines.push(Line::from(Span::styled(
                "Required env vars:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for ev in &required {
                lines.extend(render_env_var(ev, env_vars, self.reveal_secret));
            }
            lines.push(Line::from(""));
        }

        if !optional.is_empty() {
            lines.push(Line::from(Span::styled(
                "Optional env vars:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for ev in &optional {
                lines.extend(render_env_var(ev, env_vars, self.reveal_secret));
            }
        }

        // Key hints at the bottom
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(" toggle MCP  "),
            Span::styled("e", Style::default().fg(Color::Cyan)),
            Span::raw(" edit .env  "),
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" reveal secrets  "),
            Span::styled("F5", Style::default().fg(Color::Cyan)),
            Span::raw(" refresh health"),
        ]));

        let block = Block::default()
            .title(format!(" {} ", sanitize_display(plugin.name, 30)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

/// An env var descriptor used for rendering (name, description, secret flag).
struct EnvVarDesc {
    name: &'static str,
    description: &'static str,
    example: &'static str,
    secret: bool,
}

fn render_env_var(
    ev: &EnvVarDesc,
    env_vars: &HashMap<String, String>,
    reveal: bool,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let value = env_vars.get(ev.name);
    let (val_text, val_color) = value.map_or_else(
        || {
            (
                format!("(not set — example: {})", ev.example),
                Color::DarkGray,
            )
        },
        |v| {
            let display = if ev.secret && !reveal {
                // Keep the raw value in a SecretString; only expose at the
                // single render callsite — here we show a mask instead.
                let _secret = SecretString::new(v.clone());
                "●●●●●●●●".to_string()
            } else {
                sanitize_display(v, 60)
            };
            (display, Color::Green)
        },
    );

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}", ev.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" = "),
        Span::styled(val_text, Style::default().fg(val_color)),
    ]));
    if !ev.description.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("    {}", sanitize_display(ev.description, 70)),
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines
}

/// Return the required env vars for a compiled-in service by name.
/// Uses feature-gated arms to read from each service's `META` constant.
fn plugin_required_env(name: &str) -> Vec<EnvVarDesc> {
    meta_env_vars(name, true)
}

/// Return the optional env vars for a compiled-in service by name.
fn plugin_optional_env(name: &str) -> Vec<EnvVarDesc> {
    meta_env_vars(name, false)
}

fn meta_env_vars(name: &str, required: bool) -> Vec<EnvVarDesc> {
    macro_rules! collect_env {
        ($meta:expr) => {{
            let src = if required {
                $meta.required_env
            } else {
                $meta.optional_env
            };
            src.iter()
                .map(|ev| EnvVarDesc {
                    name: ev.name,
                    description: ev.description,
                    example: ev.example,
                    secret: ev.secret,
                })
                .collect()
        }};
    }

    match name {
        "extract" => collect_env!(lab_apis::extract::META),
        #[cfg(feature = "radarr")]
        "radarr" => collect_env!(lab_apis::radarr::META),
        #[cfg(feature = "sonarr")]
        "sonarr" => collect_env!(lab_apis::sonarr::META),
        #[cfg(feature = "prowlarr")]
        "prowlarr" => collect_env!(lab_apis::prowlarr::META),
        #[cfg(feature = "overseerr")]
        "overseerr" => collect_env!(lab_apis::overseerr::META),
        #[cfg(feature = "plex")]
        "plex" => collect_env!(lab_apis::plex::META),
        #[cfg(feature = "tautulli")]
        "tautulli" => collect_env!(lab_apis::tautulli::META),
        #[cfg(feature = "sabnzbd")]
        "sabnzbd" => collect_env!(lab_apis::sabnzbd::META),
        #[cfg(feature = "qbittorrent")]
        "qbittorrent" => collect_env!(lab_apis::qbittorrent::META),
        #[cfg(feature = "tailscale")]
        "tailscale" => collect_env!(lab_apis::tailscale::META),
        #[cfg(feature = "unraid")]
        "unraid" => collect_env!(lab_apis::unraid::META),
        #[cfg(feature = "unifi")]
        "unifi" => collect_env!(lab_apis::unifi::META),
        #[cfg(feature = "arcane")]
        "arcane" => collect_env!(lab_apis::arcane::META),
        #[cfg(feature = "linkding")]
        "linkding" => collect_env!(lab_apis::linkding::META),
        #[cfg(feature = "memos")]
        "memos" => collect_env!(lab_apis::memos::META),
        #[cfg(feature = "bytestash")]
        "bytestash" => collect_env!(lab_apis::bytestash::META),
        #[cfg(feature = "paperless")]
        "paperless" => collect_env!(lab_apis::paperless::META),
        #[cfg(feature = "gotify")]
        "gotify" => collect_env!(lab_apis::gotify::META),
        #[cfg(feature = "apprise")]
        "apprise" => collect_env!(lab_apis::apprise::META),
        #[cfg(feature = "openai")]
        "openai" => collect_env!(lab_apis::openai::META),
        #[cfg(feature = "notebooklm")]
        "notebooklm" => collect_env!(lab_apis::notebooklm::META),
        #[cfg(feature = "qdrant")]
        "qdrant" => collect_env!(lab_apis::qdrant::META),
        #[cfg(feature = "tei")]
        "tei" => collect_env!(lab_apis::tei::META),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::seed_enabled_services;

    #[test]
    fn empty_mcp_json_returns_empty_set_without_panic() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"").unwrap();
        let result = seed_enabled_services(Some(tmp.path()));
        assert!(
            result.is_empty(),
            "expected empty set for zero-byte .mcp.json"
        );
    }

    #[test]
    fn missing_mcp_json_returns_empty_set() {
        let result = seed_enabled_services(None);
        assert!(result.is_empty());
    }

    #[test]
    fn parses_services_from_valid_mcp_json() {
        let content = r#"{
  "mcpServers": {
    "lab": {
      "command": "/usr/local/bin/labby",
      "args": ["mcp", "--services", "radarr", "sonarr"]
    }
  }
}"#;
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        let result = seed_enabled_services(Some(tmp.path()));
        assert!(result.contains("radarr"));
        assert!(result.contains("sonarr"));
        assert_eq!(result.len(), 2);
    }
}
