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
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Terminal;

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
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        if tx_tick.send(AppEvent::Tick).is_err() {
            break;
        }
    });

    tui_main(rx)
}

// ── Main render loop ──────────────────────────────────────────────────────────

fn tui_main(rx: mpsc::Receiver<AppEvent>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    // Initial render.
    terminal.draw(|f| ui(f, &app))?;
    app.dirty = false;

    while let Ok(ev) = rx.recv() {
        handle_event(&mut app, ev);

        if app.should_quit {
            break;
        }

        if app.dirty {
            terminal.draw(|f| ui(f, &app))?;
            app.dirty = false;
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
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
            app.marketplace.preview =
                Some(crate::tui::preview::PreviewState::Ready { plugin: ready.plugin });
            app.dirty = true;
        }
        AppEvent::HealthChecksDone(_results) => {
            // Will be wired up in lab-iuk.3.
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
        (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
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
                    app.services.selected = app.services.selected.saturating_add(1);
                }
                Tab::Plugins => {
                    let len = app.marketplace.plugins.len();
                    if len > 0 {
                        app.marketplace.selected =
                            (app.marketplace.selected + 1).min(len - 1);
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
                    app.services.selected = app.services.selected.saturating_sub(1);
                }
                Tab::Plugins => {
                    app.marketplace.selected =
                        app.marketplace.selected.saturating_sub(1);
                }
                Tab::Update => {}
            }
            app.dirty = true;
        }

        // Enter / Esc — stubs for future waves
        (_, KeyCode::Enter) | (_, KeyCode::Esc) => {
            app.dirty = true;
        }

        _ => {}
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render the full TUI frame.
fn ui(f: &mut Frame<'_>, app: &App) {
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

fn render_content(f: &mut Frame<'_>, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Services => render_services(f, app, area),
        Tab::Plugins => render_plugins(f, app, area),
        Tab::Update => render_update(f, app, area),
    }
}

fn render_services(f: &mut Frame<'_>, app: &App, area: Rect) {
    let plugins = crate::tui::metadata::all_plugins();
    let content = if plugins.is_empty() {
        "No services compiled in.".to_owned()
    } else {
        plugins
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let marker = if i == app.services.selected { "> " } else { "  " };
                format!(
                    "{marker}[{cat}] {name} — {desc}",
                    cat = p.category,
                    name = p.name,
                    desc = p.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Services ");
    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}

fn render_plugins(f: &mut Frame<'_>, app: &App, area: Rect) {
    let content = if app.marketplace.loading {
        spinner_frame(app.tick_count).to_owned() + " Loading marketplace\u{2026}"
    } else if app.marketplace.plugins.is_empty() {
        "No plugins loaded. Press <Enter> to fetch marketplace catalog.".to_owned()
    } else {
        app.marketplace
            .plugins
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let marker = if i == app.marketplace.selected { "> " } else { "  " };
                format!("{marker}{} ({})", p.name, p.ecosystem.as_str())
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Plugins ");
    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}

fn render_update(f: &mut Frame<'_>, app: &App, area: Rect) {
    use crate::tui::update::UpdateState;

    let content = match &app.update {
        UpdateState::Idle => format!(
            "Current version: {}\nPress <Enter> to check for updates.",
            env!("CARGO_PKG_VERSION")
        ),
        UpdateState::Checking => {
            format!("{} Checking for updates\u{2026}", spinner_frame(app.tick_count))
        }
        UpdateState::Available { current, latest } => {
            format!("Update available: {current} \u{2192} {latest}\nPress <Enter> to download.")
        }
        UpdateState::Downloading { progress } => {
            format!(
                "{} Downloading\u{2026} {:.0}%",
                spinner_frame(app.tick_count),
                progress * 100.0
            )
        }
        UpdateState::Verifying => {
            format!("{} Verifying\u{2026}", spinner_frame(app.tick_count))
        }
        UpdateState::Done => "Up to date.".to_owned(),
        UpdateState::Error { message } => format!("Error: {message}"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Update ");
    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
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
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" back  "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" quit"),
    ]);
    f.render_widget(Paragraph::new(hints), area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const SPINNER_FRAMES: [&str; 8] = ["\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}"];

fn spinner_frame(tick: u64) -> &'static str {
    SPINNER_FRAMES[(tick as usize) % SPINNER_FRAMES.len()]
}
