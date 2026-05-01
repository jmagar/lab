//! Secret masking for `setup.draft.get` responses.
//!
//! Looks up each key against the cached secret-key set built from the
//! runtime registry. Secret values are replaced with [`SECRET_SENTINEL`]
//! before they leave the dispatch layer.

use lab_apis::setup::SECRET_SENTINEL;

use super::client::cached_secret_keys;

/// Returns `true` when `key` is registered as a secret env var on any
/// PluginMeta in the registry.
#[must_use]
pub fn is_secret_key(key: &str) -> bool {
    cached_secret_keys().contains(key)
}

/// Replace the value with the sentinel if the key is registered as a secret.
#[must_use]
pub fn mask_value(key: &str, value: &str) -> String {
    if value.is_empty() {
        return value.to_owned();
    }
    if is_secret_key(key) {
        SECRET_SENTINEL.to_owned()
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_keys_are_not_masked() {
        assert_eq!(mask_value("NOT_A_REAL_KEY", "hello"), "hello");
    }

    #[test]
    fn empty_value_passes_through() {
        // Even for known secrets, an empty value should not become "***".
        // The wizard relies on empty as "unset".
        assert_eq!(mask_value("ANYTHING", ""), "");
    }
}
