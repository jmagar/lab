//! Per-plugin install task state for the TUI progress pane.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

// ── PluginInstallState ────────────────────────────────────────────────────────

/// Tracks progress output and completion state for one plugin install/remove task.
pub struct PluginInstallState {
    /// Canonical plugin identifier (e.g. `"owner/plugin-name"`).
    pub plugin_id: String,
    /// Streamed progress lines, capped at 1 000 entries (oldest evicted first).
    pub lines: Vec<String>,
    /// `true` when the task finished successfully.
    pub done: bool,
    /// Set to the error message when the task finished with a failure.
    pub error: Option<String>,
}

impl PluginInstallState {
    /// Create a fresh state for a plugin that is about to be installed/removed.
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            lines: Vec::new(),
            done: false,
            error: None,
        }
    }

    /// Append a progress line, evicting the oldest when the cap of 1 000 is reached.
    pub fn push_line(&mut self, line: impl Into<String>) {
        if self.lines.len() >= 1_000 {
            self.lines.remove(0);
        }
        self.lines.push(line.into());
    }

    /// Mark the task as successfully completed.
    pub fn mark_done(&mut self) {
        self.done = true;
        self.error = None;
    }

    /// Mark the task as failed with the given message.
    pub fn mark_error(&mut self, message: impl Into<String>) {
        self.done = false;
        self.error = Some(message.into());
    }

    /// Render a scrolling progress pane into `area`.
    ///
    /// Shows:
    /// - A bordered block titled with the plugin id and a status badge.
    /// - The last N lines that fit in the available height, newest at the bottom.
    pub fn render(&self, f: &mut Frame<'_>, area: Rect) {
        let status_badge = if self.error.is_some() {
            Span::styled(
                " [ERROR] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else if self.done {
            Span::styled(
                " [DONE] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" [running…] ", Style::default().fg(Color::Yellow))
        };

        let title = Line::from(vec![
            Span::raw(" Install: "),
            Span::styled(
                crate::tui::display::sanitize_display(&self.plugin_id, 64),
                Style::default().fg(Color::Cyan),
            ),
            status_badge,
        ]);

        let block = Block::default().borders(Borders::ALL).title(title);

        // Inner height: subtract 2 for top+bottom borders.
        let inner_height = area.height.saturating_sub(2) as usize;

        // Show the last `inner_height` lines (tail).
        let tail_start = self.lines.len().saturating_sub(inner_height);
        let visible_lines: Vec<&String> = self.lines[tail_start..].iter().collect();

        // Append error line if present.
        let error_line_owned;
        let mut items: Vec<ListItem<'_>> = visible_lines
            .iter()
            .map(|l| {
                let safe = crate::tui::display::sanitize_display(l, 4096);
                ListItem::new(Line::from(Span::styled(
                    safe,
                    Style::default().fg(Color::White),
                )))
            })
            .collect();

        if let Some(err) = &self.error {
            error_line_owned = format!("Error: {err}");
            items.push(ListItem::new(Line::from(Span::styled(
                crate::tui::display::sanitize_display(&error_line_owned, 4096),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))));
        }

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_cap_evicts_oldest() {
        let mut state = PluginInstallState::new("owner/plugin");
        for i in 0..1_002usize {
            state.push_line(format!("line {i}"));
        }
        assert_eq!(state.lines.len(), 1_000, "must cap at 1 000 lines");
        assert_eq!(state.lines[0], "line 2", "oldest lines must be evicted");
        assert_eq!(state.lines[999], "line 1001", "newest line must be at tail");
    }

    #[test]
    fn mark_done_clears_error() {
        let mut state = PluginInstallState::new("owner/plugin");
        state.mark_error("oops");
        state.mark_done();
        assert!(state.done);
        assert!(state.error.is_none());
    }

    #[test]
    fn mark_error_clears_done() {
        let mut state = PluginInstallState::new("owner/plugin");
        state.done = true;
        state.mark_error("failed");
        assert!(!state.done);
        assert!(state.error.is_some());
    }
}
