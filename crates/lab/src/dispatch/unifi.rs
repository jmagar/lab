//! Shared dispatch layer for the `UniFi` service.
//!
//! This module is the single owner of the `UniFi` action catalog, client
//! resolution, and dispatch routing. MCP, CLI, and API are adapters over
//! this surface-neutral layer.

mod acl;
mod catalog;
mod client;
mod clients;
mod devices;
mod dispatch;
mod dns;
mod firewall;
mod hotspot;
mod misc;
mod networks;
mod params;
mod switching;
mod traffic;
mod wifi;

pub use catalog::actions;
pub use client::client_from_env;
pub use dispatch::dispatch;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::error::ToolError;

    #[tokio::test]
    async fn unifi_service_help_lists_core_actions() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "system.info"));
        assert!(actions.iter().any(|a| a["name"] == "sites.list"));
        assert!(actions.iter().any(|a| a["name"] == "clients.list"));
        assert!(actions.iter().any(|a| a["name"] == "wifi.broadcasts.list"));
    }

    #[test]
    fn action_count_matches_original() {
        assert_eq!(
            actions().len(),
            72,
            "expected 72 actions (help + 71 resource actions), got {}",
            actions().len()
        );
    }

    #[test]
    fn no_duplicate_action_names() {
        let names: Vec<&str> = actions().iter().map(|a| a.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            names.len(),
            sorted.len(),
            "duplicate action names found in ACTIONS"
        );
    }

    #[tokio::test]
    async fn unknown_action_includes_valid_list() {
        let err = dispatch("bogus.action", serde_json::json!({}))
            .await
            .unwrap_err();
        match err {
            ToolError::UnknownAction { valid, .. } => {
                assert!(valid.iter().any(|a| a == "help"));
                assert!(valid.iter().any(|a| a == "system.info"));
            }
            other => panic!("expected unknown action error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn devices_list_is_routed_by_service_layer() {
        let err = dispatch("devices.list", serde_json::json!({"site_id": "default"}))
            .await
            .unwrap_err();
        assert_ne!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn firewall_zones_list_is_routed_by_service_layer() {
        let err = dispatch(
            "firewall.zones.list",
            serde_json::json!({"site_id": "default"}),
        )
        .await
        .unwrap_err();
        assert_ne!(err.kind(), "unknown_action");
    }
}
