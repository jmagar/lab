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

const SPINNER_FRAMES: [&str; 8] = [
    "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}",
];

/// Return the spinner frame for the given tick count.
/// Uses 8-frame braille spinner cycling at tick rate.
pub fn spinner_frame(tick: u64) -> &'static str {
    SPINNER_FRAMES[(tick as usize) % SPINNER_FRAMES.len()]
}
