//! Read/write `.env.draft` via the shared `env_merge` primitive.

use std::fs;
use std::path::Path;

use lab_apis::setup::DraftEntry;

use crate::config::env_merge::{
    self, EnvEntry, MergeError, MergeOutcome, MergeRequest, strip_quotes,
};

/// Parse an `.env`-style file into `(key, value)` entries. Comments and
/// blank lines are dropped; quoted values are unwrapped.
#[must_use]
pub fn read_entries(path: &Path) -> Vec<DraftEntry> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (k, v) = trimmed.split_once('=')?;
            Some(DraftEntry {
                key: k.trim().to_string(),
                value: strip_quotes(v.trim()),
            })
        })
        .collect()
}

/// Merge `entries` into `path` (typically `.env.draft`).
pub fn merge_entries(
    path: &Path,
    entries: Vec<DraftEntry>,
    force: bool,
) -> Result<MergeOutcome, MergeError> {
    env_merge::merge(
        path,
        MergeRequest {
            entries: entries
                .into_iter()
                .map(|e| EnvEntry::new(e.key, e.value))
                .collect(),
            force,
            expected_mtime: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_quoted_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env.draft");
        merge_entries(
            &path,
            vec![
                DraftEntry {
                    key: "FOO".into(),
                    value: "bar baz".into(),
                },
                DraftEntry {
                    key: "BAZ".into(),
                    value: "qux".into(),
                },
            ],
            false,
        )
        .unwrap();
        let entries = read_entries(&path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "FOO");
        assert_eq!(entries[0].value, "bar baz");
        assert_eq!(entries[1].key, "BAZ");
        assert_eq!(entries[1].value, "qux");
    }
}
