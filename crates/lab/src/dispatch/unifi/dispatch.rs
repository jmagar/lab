use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::{
    acl, catalog, clients, devices, dns, firewall, hotspot, misc, networks, switching, traffic,
    wifi,
};

/// Dispatch one call against the `UniFi` service.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("unifi", catalog::actions())),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            action_schema(catalog::actions(), action_name)
        }
        a if a.starts_with("devices.") || a == "pending-devices.list" => {
            devices::dispatch(action, params).await
        }
        a if a.starts_with("clients.") => clients::dispatch(action, params).await,
        a if a.starts_with("networks.") || a.starts_with("network.") => {
            networks::dispatch(action, params).await
        }
        a if a.starts_with("wifi.") => wifi::dispatch(action, params).await,
        a if a.starts_with("hotspot.") => hotspot::dispatch(action, params).await,
        a if a.starts_with("firewall.") => firewall::dispatch(action, params).await,
        a if a.starts_with("acl.") => acl::dispatch(action, params).await,
        a if a.starts_with("switching.") => switching::dispatch(action, params).await,
        a if a.starts_with("dns.") => dns::dispatch(action, params).await,
        a if a.starts_with("traffic-matching-lists.") => traffic::dispatch(action, params).await,
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
            valid: catalog::actions()
                .iter()
                .map(|a| a.name.to_string())
                .collect(),
            hint: None,
        }),
    }
}
