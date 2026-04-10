//! Core application state for the TUI plugin manager.

use std::collections::VecDeque;

use crate::tui::marketplace::MarketplaceState;
use crate::tui::services::LabServicesState;
use crate::tui::update::UpdateState;

/// Top-level tab selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Services,
    Plugins,
    Update,
}

/// Ecosystem / AI agent platform variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    ClaudeCode,
    Codex,
    Gemini,
}

impl Ecosystem {
    /// Return the kebab-case identifier used in config paths and CLI commands.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }

    /// Return the human-readable display name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini CLI",
        }
    }

    /// All ecosystem variants — used to drive match exhaustiveness checks and UI lists.
    pub const ALL: [Ecosystem; 3] = [Self::ClaudeCode, Self::Codex, Self::Gemini];
}

/// A transient notification message shown at the bottom of the screen.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
}

/// Root application state. Passed mutably through the event loop.
pub struct App {
    pub current_tab: Tab,
    pub services: LabServicesState,
    pub marketplace: MarketplaceState,
    pub update: UpdateState,
    pub toasts: VecDeque<Toast>,
    /// Set to `true` whenever state changes; cleared after each render pass.
    pub dirty: bool,
    /// Incremented on every `AppEvent::Tick`; drives spinner animation.
    pub tick_count: u64,
    /// Set to `true` when the user requests exit.
    pub should_quit: bool,
    /// Set to `true` when the user presses `e` to open `~/.lab/.env` in `$EDITOR`.
    /// `tui_main` handles this by tearing down the terminal, launching the editor,
    /// re-initializing the terminal, and reloading the env cache.
    pub open_editor: bool,
    /// Set to `true` when the user presses `F5` to re-run health checks.
    /// `tui_main` spawns a fresh background health task and clears this flag.
    pub refresh_health: bool,
    /// Set to `true` while a background health check is in flight.
    /// Guards against spawning duplicate health check tasks.
    pub health_check_in_flight: bool,
}

impl App {
    /// Create a fresh application state.
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Services,
            services: LabServicesState::default(),
            marketplace: MarketplaceState::default(),
            update: UpdateState::default(),
            toasts: VecDeque::new(),
            dirty: true,
            tick_count: 0,
            should_quit: false,
            open_editor: false,
            refresh_health: false,
            health_check_in_flight: false,
        }
    }

    /// Append a toast message, evicting the oldest if the queue is full.
    pub fn push_toast(&mut self, message: impl Into<String>) {
        if self.toasts.len() >= 5 {
            self.toasts.pop_front();
        }
        self.toasts.push_back(Toast {
            message: message.into(),
        });
        self.dirty = true;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
