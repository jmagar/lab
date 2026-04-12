//! MCP adapter for the `UniFi` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/unifi.rs`.

#[cfg(test)]
mod tests {
    #[test]
    fn help_lists_read_only_actions() {
        let actions: Vec<_> = crate::dispatch::unifi::actions()
            .iter()
            .map(|a| a.name)
            .collect();
        assert!(actions.contains(&"help"));
        assert!(actions.contains(&"system.info"));
        assert!(actions.contains(&"sites.list"));
        assert!(actions.contains(&"clients.list"));
        assert!(actions.contains(&"networks.list"));
        assert!(actions.contains(&"wifi.broadcasts.list"));
    }

    #[test]
    fn action_count_matches_service_layer() {
        assert_eq!(crate::dispatch::unifi::actions().len(), 72);
    }
}
