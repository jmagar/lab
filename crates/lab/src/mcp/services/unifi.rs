//! MCP dispatch for the `UniFi` tool.

use serde_json::Value;

use lab_apis::core::Auth;
use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::unifi::UnifiClient;
use lab_apis::unifi::error::UnifiError;

use crate::mcp::envelope::ToolError;

impl From<UnifiError> for ToolError {
    fn from(e: UnifiError) -> Self {
        let kind = match &e {
            UnifiError::Api(api) => api.kind(),
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Action catalog for the `UniFi` tool.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "system.info",
        description: "Return application version and runtime metadata",
        destructive: false,
        returns: "ApplicationInfo",
        params: &[],
    },
    ActionSpec {
        name: "sites.list",
        description: "List local `UniFi` sites",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "devices.list",
        description: "List adopted devices for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "devices.get",
        description: "Inspect one adopted device",
        destructive: false,
        returns: "Device",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
        ],
    },
    ActionSpec {
        name: "devices.stats",
        description: "Get latest statistics for one adopted device",
        destructive: false,
        returns: "DeviceStats",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
        ],
    },
    ActionSpec {
        name: "pending-devices.list",
        description: "List devices pending adoption",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "clients.list",
        description: "List connected clients for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "clients.get",
        description: "Inspect one connected client",
        destructive: false,
        returns: "Client",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "client_id",
                ty: "string",
                required: true,
                description: "Client UUID",
            },
        ],
    },
    ActionSpec {
        name: "networks.list",
        description: "List networks for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "network.get",
        description: "Inspect one network",
        destructive: false,
        returns: "Network",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
    ActionSpec {
        name: "network.references",
        description: "Get references for one network",
        destructive: false,
        returns: "ReferenceResources",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
    ActionSpec {
        name: "wifi.broadcasts.list",
        description: "List `WiFi` broadcasts for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "wifi.broadcasts.get",
        description: "Inspect one `WiFi` broadcast",
        destructive: false,
        returns: "WifiBroadcast",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "wifi_broadcast_id",
                ty: "string",
                required: true,
                description: "`WiFi` broadcast UUID",
            },
        ],
    },
];

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = std::env::var("UNIFI_URL").ok()?;
    let key = std::env::var("UNIFI_API_KEY").ok()?;
    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .ok()
}

fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}

fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Dispatch one MCP call against the `UniFi` tool.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "unifi",
            "actions": ACTIONS.iter().map(|a| serde_json::json!({
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
        "system.info" => {
            let info = require_client()?.info().await?;
            to_json(info)
        }
        "sites.list" => {
            let sites = require_client()?.sites_list().await?;
            to_json(sites)
        }
        "devices.list" => {
            let site_id = require_str(&params, "site_id")?;
            let devices = require_client()?.devices_list(site_id).await?;
            to_json(devices)
        }
        "devices.get" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let device = require_client()?.device_get(site_id, device_id).await?;
            to_json(device)
        }
        "devices.stats" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let stats = require_client()?
                .device_stats_latest(site_id, device_id)
                .await?;
            to_json(stats)
        }
        "pending-devices.list" => {
            let pending = require_client()?.pending_devices_list().await?;
            to_json(pending)
        }
        "clients.list" => {
            let site_id = require_str(&params, "site_id")?;
            let clients = require_client()?.clients_list(site_id).await?;
            to_json(clients)
        }
        "clients.get" => {
            let site_id = require_str(&params, "site_id")?;
            let client_id = require_str(&params, "client_id")?;
            let client = require_client()?.client_get(site_id, client_id).await?;
            to_json(client)
        }
        "networks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let networks = require_client()?.networks_list(site_id).await?;
            to_json(networks)
        }
        "network.get" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let network = require_client()?.network_get(site_id, network_id).await?;
            to_json(network)
        }
        "network.references" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let refs = require_client()?
                .network_references(site_id, network_id)
                .await?;
            to_json(refs)
        }
        "wifi.broadcasts.list" => {
            let site_id = require_str(&params, "site_id")?;
            let broadcasts = require_client()?.wifi_broadcasts_list(site_id).await?;
            to_json(broadcasts)
        }
        "wifi.broadcasts.get" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            let broadcast = require_client()?
                .wifi_broadcast_get(site_id, wifi_broadcast_id)
                .await?;
            to_json(broadcast)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `unifi.{unknown}`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_lists_read_only_actions() {
        let actions: Vec<_> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(actions.contains(&"help"));
        assert!(actions.contains(&"system.info"));
        assert!(actions.contains(&"sites.list"));
        assert!(actions.contains(&"clients.list"));
        assert!(actions.contains(&"networks.list"));
        assert!(actions.contains(&"wifi.broadcasts.list"));
    }
}
