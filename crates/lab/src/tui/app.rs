//! TUI entry point — channel-based event loop (Pattern B).
//!
//! Architecture:
//! - Thread 1: crossterm input poller (non-blocking, 5 ms sleep)
//! - Thread 2: tick generator (100 ms interval, drives spinner)
//! - Tokio runtime: background async tasks post events on the same channel
//! - Main thread: render loop — blocks on `rx.recv()`, renders only when dirty

use std::io::Stderr;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

use crate::tui::events::AppEvent;
use crate::tui::state::{App, Tab};

// ── Terminal lifecycle ────────────────────────────────────────────────────────

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stderr>>> {
    let mut stderr = std::io::stderr();
    enable_raw_mode()?;
    execute!(stderr, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stderr))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) {
    drop(disable_raw_mode());
    drop(execute!(terminal.backend_mut(), LeaveAlternateScreen));
    drop(terminal.show_cursor());
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Run the plugin manager TUI.
pub fn run() -> Result<()> {
    // Install panic hook BEFORE touching the terminal so panics restore it.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        drop(disable_raw_mode());
        drop(execute!(std::io::stderr(), LeaveAlternateScreen));
        original_hook(info);
    }));

    // Handle to the ambient tokio runtime (created by #[tokio::main] in main.rs).
    // Do NOT create a new Runtime here — dropping a Runtime inside an async context panics.
    // Use this handle to spawn background tasks (marketplace load, health checks, etc.).
    let _rt_handle = tokio::runtime::Handle::current();

    let (tx, rx) = mpsc::channel::<AppEvent>();

    // Thread 1 — crossterm input poller (Pattern B: non-blocking poll + sleep).
    let tx_input = tx.clone();
    std::thread::spawn(move || {
        loop {
            match event::poll(Duration::ZERO) {
                Ok(true) => {
                    if let Ok(ev) = event::read() {
                        if tx_input.send(AppEvent::Input(ev)).is_err() {
                            break;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => break,
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });

    // Thread 2 — tick generator (100 ms, drives spinner animation).
    let tx_tick = tx.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(100));
            if tx_tick.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    tui_main(tx, rx)
}

// ── Main render loop ──────────────────────────────────────────────────────────

fn tui_main(tx: mpsc::Sender<AppEvent>, rx: mpsc::Receiver<AppEvent>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    // Spawn startup background tasks.
    spawn_health_check(tx.clone());
    spawn_marketplace_load(tx.clone());

    // Initial render.
    terminal.draw(|f| ui(f, &mut app))?;
    app.dirty = false;

    while let Ok(ev) = rx.recv() {
        handle_event(&mut app, ev);

        if app.should_quit {
            break;
        }

        // Handle editor launch: tear down terminal, run editor, re-init, reload cache.
        if app.open_editor {
            app.open_editor = false;
            restore_terminal(&mut terminal);
            let _ = crate::tui::services::LabServicesState::open_env_editor();
            match setup_terminal() {
                Ok(t) => {
                    terminal = t;
                    app.services.reload_env_cache();
                    app.dirty = true;
                }
                Err(e) => {
                    app.push_toast(format!("Terminal re-init failed: {e}"));
                    app.should_quit = true;
                }
            }
        }

        // Handle health refresh: spawn a fresh health check background task.
        if app.refresh_health {
            app.refresh_health = false;
            spawn_health_check(tx.clone());
        }

        if app.dirty {
            terminal.draw(|f| ui(f, &mut app))?;
            app.dirty = false;
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}

/// Spawn a background task that runs health checks for all enabled services
/// and posts the results back on the event channel.
fn spawn_health_check(tx: mpsc::Sender<AppEvent>) {
    let env_path = crate::tui::services::lab_env_path();
    tokio::runtime::Handle::current().spawn(async move {
        let results = crate::tui::metadata::check_all_services(&env_path).await;
        let _ = tx.send(AppEvent::HealthChecksDone(results));
    });
}

/// Spawn a background task that detects installed CLIs then loads all marketplace
/// catalogs, posting the result back on the event channel.
fn spawn_marketplace_load(tx: mpsc::Sender<AppEvent>) {
    tokio::runtime::Handle::current().spawn(async move {
        let cli = crate::tui::marketplace::CliPresence::detect().await;
        let plugins = crate::tui::marketplace::MarketplaceLoader::load_all(&cli).await;
        let _ = tx.send(AppEvent::MarketplaceLoaded(plugins));
    });
}

// ── Event handling ────────────────────────────────────────────────────────────

fn handle_event(app: &mut App, ev: AppEvent) {
    match ev {
        AppEvent::Input(Event::Key(key)) => handle_key(app, key),
        AppEvent::Input(_) => {} // resize / mouse — ignore for now
        AppEvent::Tick => {
            app.tick_count = app.tick_count.wrapping_add(1);
            app.dirty = true;
        }
        AppEvent::TaskError { message, .. } => {
            app.push_toast(message);
        }
        AppEvent::MarketplaceLoaded(plugins) => {
            app.marketplace.plugins = plugins;
            app.marketplace.loading = false;
            app.dirty = true;
        }
        AppEvent::UpdateCheckDone { latest, .. } => {
            if let Some(latest_ver) = latest {
                use crate::tui::update::UpdateState;
                app.update = UpdateState::Available {
                    current: env!("CARGO_PKG_VERSION").to_owned(),
                    latest: latest_ver,
                };
            } else {
                app.update = crate::tui::update::UpdateState::Done;
            }
            app.dirty = true;
        }
        AppEvent::PreviewReady(ready) => {
            app.marketplace.preview = Some(crate::tui::preview::PreviewState::Ready {
                plugin: ready.plugin,
            });
            app.dirty = true;
        }
        AppEvent::HealthChecksDone(results) => {
            app.services.update_health(results);
            app.dirty = true;
        }
        AppEvent::ProgressLine { .. } | AppEvent::TaskDone { .. } => {
            app.dirty = true;
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        // Quit
        (_, KeyCode::Char('q'))
        | (_, KeyCode::Esc)
        | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.should_quit = true;
        }

        // Tab switching — Tab / Shift+Tab cycle
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.current_tab = match app.current_tab {
                Tab::Services => Tab::Plugins,
                Tab::Plugins => Tab::Update,
                Tab::Update => Tab::Services,
            };
            app.dirty = true;
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            app.current_tab = match app.current_tab {
                Tab::Services => Tab::Update,
                Tab::Plugins => Tab::Services,
                Tab::Update => Tab::Plugins,
            };
            app.dirty = true;
        }

        // Tab switching — numeric shortcuts
        (_, KeyCode::Char('1')) => {
            app.current_tab = Tab::Services;
            app.dirty = true;
        }
        (_, KeyCode::Char('2')) => {
            app.current_tab = Tab::Plugins;
            app.dirty = true;
        }
        (_, KeyCode::Char('3')) => {
            app.current_tab = Tab::Update;
            app.dirty = true;
        }

        // Navigation — down
        (_, KeyCode::Char('j')) | (_, KeyCode::Down) => {
            match app.current_tab {
                Tab::Services => {
                    app.services.select_next();
                }
                Tab::Plugins => {
                    let len = app.marketplace.plugins.len();
                    if len > 0 {
                        app.marketplace.selected = (app.marketplace.selected + 1).min(len - 1);
                    }
                }
                Tab::Update => {}
            }
            app.dirty = true;
        }

        // Navigation — up
        (_, KeyCode::Char('k')) | (_, KeyCode::Up) => {
            match app.current_tab {
                Tab::Services => {
                    app.services.select_prev();
                }
                Tab::Plugins => {
                    app.marketplace.selected = app.marketplace.selected.saturating_sub(1);
                }
                Tab::Update => {}
            }
            app.dirty = true;
        }

        // Enter — stub for future waves
        (_, KeyCode::Enter) => {
            app.dirty = true;
        }

        // Space — toggle service in .mcp.json (Services tab only)
        (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            if app.current_tab == Tab::Services {
                if let Err(e) = app.services.toggle_enabled() {
                    app.push_toast(format!("Toggle failed: {e}"));
                }
                app.dirty = true;
            }
        }

        // e — open ~/.lab/.env in $EDITOR (Services tab only)
        (KeyModifiers::NONE, KeyCode::Char('e')) => {
            if app.current_tab == Tab::Services {
                app.open_editor = true;
            }
        }

        // r — toggle secret reveal (Services tab only)
        (KeyModifiers::NONE, KeyCode::Char('r')) => {
            if app.current_tab == Tab::Services {
                app.services.toggle_reveal();
                app.dirty = true;
            }
        }

        // F5 — refresh health dots (Services tab only)
        (_, KeyCode::F(5)) => {
            if app.current_tab == Tab::Services {
                app.refresh_health = true;
                app.push_toast("Refreshing health\u{2026}".to_string());
            }
        }

        _ => {}
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render the full TUI frame.
fn ui(f: &mut Frame<'_>, app: &mut App) {
    let area = f.area();

    // Vertical layout: tab bar | content | toasts | key hints
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(1),    // content
            Constraint::Length(1), // toasts
            Constraint::Length(1), // key hints
        ])
        .split(area);

    render_tabs(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_toasts(f, app, chunks[2]);
    render_hints(f, chunks[3]);
}

fn render_tabs(f: &mut Frame<'_>, app: &App, area: Rect) {
    let tab_titles = vec![
        Line::from("1 Services"),
        Line::from("2 Plugins"),
        Line::from("3 Update"),
    ];
    let selected = match app.current_tab {
        Tab::Services => 0,
        Tab::Plugins => 1,
        Tab::Update => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" lab "))
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    match app.current_tab {
        Tab::Services => render_services(f, app, area),
        Tab::Plugins => render_plugins(f, app, area),
        Tab::Update => render_update(f, app, area),
    }
}

fn render_services(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    app.services.render(f, area, app.tick_count);
}

fn render_plugins(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    app.marketplace.render(f, area, app.tick_count);
}

fn render_update(f: &mut Frame<'_>, app: &mut App, area: Rect) {
    app.update.render(f, area, app.tick_count);
}

fn render_toasts(f: &mut Frame<'_>, app: &App, area: Rect) {
    let text = app
        .toasts
        .iter()
        .map(|t| crate::tui::display::sanitize_display(&t.message, 120))
        .collect::<Vec<_>>()
        .join(" | ");

    let line = Line::from(Span::styled(text, Style::default().fg(Color::Yellow)));
    f.render_widget(Paragraph::new(line), area);
}

fn render_hints(f: &mut Frame<'_>, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" switch tab  "),
        Span::styled("j/k \u{2191}\u{2193}", Style::default().fg(Color::Cyan)),
        Span::raw(" navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" confirm  "),
        Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" quit"),
    ]);
    f.render_widget(Paragraph::new(hints), area);
}
