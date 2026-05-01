//! First-run detection + state-machine evaluator for `setup.state`.

use lab_apis::setup::{SetupSnapshot, SetupState};
use std::path::Path;

use crate::config::env_merge::snapshot_mtime;
use crate::registry::{ToolRegistry, service_meta};

use super::client::{draft_path, env_path};
use super::draft;

/// Read every required env var from the registry. A service contributes its
/// required vars unconditionally — wizards skip optional ones.
fn registry_required_keys(registry: &ToolRegistry) -> Vec<String> {
    let mut keys = Vec::new();
    for entry in registry.services() {
        if let Some(meta) = service_meta(entry.name) {
            for var in meta.required_env {
                keys.push(var.name.to_string());
            }
        }
    }
    keys
}

/// Build a `SetupSnapshot` describing the current state of `~/.lab/.env`.
#[must_use]
pub fn snapshot(registry: &ToolRegistry) -> SetupSnapshot {
    let env = env_path();
    let draft = draft_path();
    let env_exists = env.exists();
    let has_draft = draft.exists();
    let draft_stale = draft_is_stale(&env, &draft);

    let state = if !env_exists {
        SetupState::Uninitialized
    } else {
        let entries = draft::read_entries(&env);
        let registered: Vec<String> = registry_required_keys(registry);
        let missing: Vec<String> = registered
            .into_iter()
            .filter(|key| {
                !entries
                    .iter()
                    .any(|e| &e.key == key && !e.value.is_empty())
            })
            .collect();
        if missing.is_empty() {
            SetupState::Ready
        } else if entries.is_empty() {
            SetupState::ConfigMissing { envars: missing }
        } else {
            SetupState::PartiallyConfigured { missing }
        }
    };

    SetupSnapshot {
        first_run: matches!(
            state,
            SetupState::Uninitialized | SetupState::ConfigMissing { .. }
        ),
        env_path: env,
        draft_path: draft,
        last_completed_step: 0,
        draft_stale,
        has_draft,
        state,
    }
}

fn draft_is_stale(env: &Path, draft: &Path) -> bool {
    if !draft.exists() {
        return false;
    }
    let env_mtime = snapshot_mtime(env);
    let draft_mtime = snapshot_mtime(draft);
    match (env_mtime, draft_mtime) {
        (Some(e), Some(d)) => e > d,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    // Note: snapshot() reads LAB_HOME via env, which Rust 2024 marks unsafe.
    // The crate forbids unsafe, so we can't mutate the env var inside tests
    // here. End-to-end coverage of the state machine ships in the smoke test
    // recipe (`just smoke-setup`) added in Chunk F.
}
