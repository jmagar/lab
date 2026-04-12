use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_str};

use super::{
    acl, catalog, clients, devices, dns, firewall, hotspot, misc, networks, switching, traffic,
    wifi,
};
use lab_apis::unifi::UnifiClient;

/// Dispatch one call against the `UniFi` service.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    let actions = catalog::actions();
    match action {
        "help" => return Ok(help_payload("unifi", actions)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(actions, action_name);
        }
        _ => {}
    }

    if !actions.iter().any(|spec| spec.name == action) {
        return Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unifi`"),
            valid: actions.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        });
    }

    let instance = optional_str(&params, "instance")?.map(str::to_owned);
    let mut params = params;
    if let Value::Object(ref mut map) = params {
        map.remove("instance");
    }

    match instance {
        Some(label) => {
            let client = super::client::client_from_instance(&label)?;
            dispatch_with_client(&client, action, params).await
        }
        None => {
            let client = super::client::require_client()?;
            dispatch_with_client(&client, action, params).await
        }
    }
}

/// Dispatch one `UniFi` call with an explicit client.
pub async fn dispatch_with_client(
    client: &UnifiClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        a if a.starts_with("devices.") || a == "pending-devices.list" => {
            devices::dispatch(client, action, params).await
        }
        a if a.starts_with("clients.") => clients::dispatch(client, action, params).await,
        a if a.starts_with("networks.") => {
            networks::dispatch(client, action, params).await
        }
        a if a.starts_with("wifi.") => wifi::dispatch(client, action, params).await,
        a if a.starts_with("hotspot.") => hotspot::dispatch(client, action, params).await,
        a if a.starts_with("firewall.") => firewall::dispatch(client, action, params).await,
        a if a.starts_with("acl.") => acl::dispatch(client, action, params).await,
        a if a.starts_with("switching.") => switching::dispatch(client, action, params).await,
        a if a.starts_with("dns.") => dns::dispatch(client, action, params).await,
        a if a.starts_with("traffic-matching-lists.") => {
            traffic::dispatch(client, action, params).await
        }
        "system.info" | "sites.list" | "wans.list" | "countries.list" => {
            misc::dispatch(client, action, params).await
        }
        a if a.starts_with("vpn.")
            || a.starts_with("radius.")
            || a.starts_with("device-tags.")
            || a.starts_with("dpi.") =>
        {
            misc::dispatch(client, action, params).await
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
