//! MCP dispatch for the `UniFi` tool.
//!
//! Split into focused sub-modules by resource group. This entry point
//! assembles the combined `ACTIONS` catalog and delegates dispatch to
//! the appropriate sub-module.

mod acl;
mod clients;
mod devices;
mod dns;
mod firewall;
pub mod helpers;
mod hotspot;
mod misc;
mod networks;
mod switching;
mod traffic;
mod wifi;

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::services::error::ToolError;

// Re-export helpers used by other modules (e.g. cli/health.rs, tui/metadata.rs).
pub use helpers::client_from_env;

/// Combined action catalog for the `UniFi` tool.
///
/// Built once at first access by concatenating each sub-module's `ACTIONS`
/// slice. The resulting `Vec` is leaked to produce a `&'static [ActionSpec]`
/// compatible with `RegisteredService::actions`.
pub fn actions() -> &'static [ActionSpec] {
    static ACTIONS: std::sync::LazyLock<&'static [ActionSpec]> =
        std::sync::LazyLock::new(|| {
            let mut all = vec![ActionSpec {
                name: "help",
                description: "Show this action catalog",
                destructive: false,
                returns: "Catalog",
                params: &[],
            }];
            all.extend_from_slice(misc::ACTIONS);
            all.extend_from_slice(devices::ACTIONS);
            all.extend_from_slice(clients::ACTIONS);
            all.extend_from_slice(networks::ACTIONS);
            all.extend_from_slice(wifi::ACTIONS);
            all.extend_from_slice(hotspot::ACTIONS);
            all.extend_from_slice(firewall::ACTIONS);
            all.extend_from_slice(acl::ACTIONS);
            all.extend_from_slice(switching::ACTIONS);
            all.extend_from_slice(dns::ACTIONS);
            all.extend_from_slice(traffic::ACTIONS);
            Vec::leak(all)
        });
    &ACTIONS
}

/// Dispatch one MCP call against the `UniFi` tool.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "unifi",
            "actions": actions().iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "returns": a.returns,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),

        // Devices & pending devices
        a if a.starts_with("devices.") || a == "pending-devices.list" => {
            devices::dispatch(action, params).await
        }

        // Clients
        a if a.starts_with("clients.") => clients::dispatch(action, params).await,

        // Networks
        a if a.starts_with("networks.") || a.starts_with("network.") => {
            networks::dispatch(action, params).await
        }

        // Wi-Fi broadcasts
        a if a.starts_with("wifi.") => wifi::dispatch(action, params).await,

        // Hotspot vouchers
        a if a.starts_with("hotspot.") => hotspot::dispatch(action, params).await,

        // Firewall zones & policies
        a if a.starts_with("firewall.") => firewall::dispatch(action, params).await,

        // ACL rules
        a if a.starts_with("acl.") => acl::dispatch(action, params).await,

        // Switching (switch stacks, MC-LAG, LAGs)
        a if a.starts_with("switching.") => switching::dispatch(action, params).await,

        // DNS policies
        a if a.starts_with("dns.") => dns::dispatch(action, params).await,

        // Traffic matching lists
        a if a.starts_with("traffic-matching-lists.") => {
            traffic::dispatch(action, params).await
        }

        // Misc (system, sites, WANs, VPN, RADIUS, device-tags, DPI, countries)
        "system.info" | "sites.list" | "wans.list" | "countries.list" => {
            misc::dispatch(action, params).await
        }
        a if a.starts_with("vpn.")
            || a.starts_with("radius.")
            || a.starts_with("device-tags.")
            || a.starts_with("dpi.") =>
        {
            misc::dispatch(action, params).await
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `unifi.{unknown}`"),
            valid: actions().iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
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
    fn action_count_matches_original() {
        // Original monolith had 72 ActionSpecs (including help).
        // Verify we haven't lost any.
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
}
