//! Minimal INI parser shared by SABnzbd, qBittorrent, and Tautulli parsers.
//!
//! Supports:
//! - `[Section]` headers (stored as lowercase)
//! - `key = value` pairs (keys stored as-is, lowercased by callers if needed)
//! - `#` and `;` line comments
//! - Blank lines ignored
//! - Keys before any section header are stored under the empty-string section

use std::collections::HashMap;

/// Parse INI bytes into `HashMap<section, HashMap<key, value>>`.
///
/// Section names are stored lowercase. Keys preserve their original case.
/// Values are stored as-is (no unquoting).
#[must_use]
pub(super) fn parse(contents: &[u8]) -> HashMap<String, HashMap<String, String>> {
    let text = String::from_utf8_lossy(contents);
    let mut out: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut section = String::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            if let Some(end) = line.find(']') {
                section = line[1..end].trim().to_lowercase();
                // Ensure the section entry exists even if it has no keys.
                out.entry(section.clone()).or_default();
            }
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            out.entry(section.clone())
                .or_default()
                .insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }

    out
}
