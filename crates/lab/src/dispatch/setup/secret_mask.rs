//! Secret masking for `setup.draft.get` responses.
//!
//! Walks the runtime registry to find the `EnvVar.secret` flag attached to
//! each key. Secret values are replaced with [`SECRET_SENTINEL`] before
//! they leave the dispatch layer.

use lab_apis::core::PluginMeta;
use lab_apis::setup::SECRET_SENTINEL;

use crate::registry::{ToolRegistry, service_meta};

/// Returns `true` when `key` is registered as a secret env var on any
/// PluginMeta in the registry.
#[must_use]
pub fn is_secret_key(registry: &ToolRegistry, key: &str) -> bool {
    for entry in registry.services() {
        let Some(meta) = service_meta(entry.name) else {
            continue;
        };
        if meta_lists_secret(meta, key) {
            return true;
        }
    }
    false
}

fn meta_lists_secret(meta: &PluginMeta, key: &str) -> bool {
    meta.required_env
        .iter()
        .chain(meta.optional_env.iter())
        .any(|var| var.name == key && var.secret)
}

/// Replace the value with the sentinel if the key is registered as a secret.
#[must_use]
pub fn mask_value(registry: &ToolRegistry, key: &str, value: &str) -> String {
    if value.is_empty() {
        return value.to_owned();
    }
    if is_secret_key(registry, key) {
        SECRET_SENTINEL.to_owned()
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::build_default_registry;

    #[test]
    fn unknown_keys_are_not_masked() {
        let reg = build_default_registry();
        assert_eq!(mask_value(&reg, "NOT_A_REAL_KEY", "hello"), "hello");
    }

    #[test]
    fn empty_value_passes_through() {
        let reg = build_default_registry();
        // Even for known secrets, an empty value should not become "***".
        // The wizard relies on empty as "unset".
        // (No assumption about which key is secret — this is a pure-input
        // test of the early return.)
        assert_eq!(mask_value(&reg, "ANYTHING", ""), "");
    }
}
