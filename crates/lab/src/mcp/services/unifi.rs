//! MCP adapter for the `UniFi` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/unifi.rs`.
//! This module forwards catalog, client resolution, and dispatch for MCP
//! wiring and compatibility with existing call sites during migration.

use lab_apis::core::action::ActionSpec;
use lab_apis::unifi::UnifiClient;

#[allow(dead_code)]
pub fn client_from_env() -> Option<UnifiClient> {
    crate::dispatch::unifi::client_from_env()
}

#[allow(dead_code)]
pub fn actions() -> &'static [ActionSpec] {
    crate::dispatch::unifi::actions()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_lists_read_only_actions() {
        let actions: Vec<_> = actions().iter().map(|a| a.name).collect();
        assert!(actions.contains(&"help"));
        assert!(actions.contains(&"system.info"));
        assert!(actions.contains(&"sites.list"));
        assert!(actions.contains(&"clients.list"));
        assert!(actions.contains(&"networks.list"));
        assert!(actions.contains(&"wifi.broadcasts.list"));
    }

    #[test]
    fn action_count_matches_service_layer() {
        assert_eq!(actions().len(), crate::dispatch::unifi::actions().len());
        assert_eq!(actions().len(), 72);
    }
}
