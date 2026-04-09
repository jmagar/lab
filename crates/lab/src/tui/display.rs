//! Display helpers shared across all TUI tabs.

/// Strip ANSI escape sequences and cap length for safe ratatui rendering.
/// All strings from marketplace JSON, subprocess output, or external sources
/// MUST pass through this before being rendered in ratatui widgets.
pub fn sanitize_display(s: &str, max_len: usize) -> String {
    let stripped = strip_ansi_escapes::strip_str(s);
    if stripped.len() <= max_len {
        stripped
    } else {
        let mut truncated = stripped[..max_len.min(stripped.len())].to_string();
        truncated.push('\u{2026}');
        truncated
    }
}
