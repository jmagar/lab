//! MCP dispatch for the `UniFi` tool.

use serde_json::{Map, Value};

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
    ActionSpec {
        name: "devices.create",
        description: "Adopt a device to a site",
        destructive: true,
        returns: "Device",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "macAddress",
                ty: "string",
                required: true,
                description: "Device MAC address",
            },
            ParamSpec {
                name: "ignoreDeviceLimit",
                ty: "bool",
                required: false,
                description: "Ignore device limit",
            },
        ],
    },
    ActionSpec {
        name: "devices.port-action",
        description: "Perform an action on one device port",
        destructive: true,
        returns: "Value",
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
            ParamSpec {
                name: "port_idx",
                ty: "i64",
                required: true,
                description: "Port index",
            },
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Port action name",
            },
        ],
    },
    ActionSpec {
        name: "devices.action",
        description: "Perform an action on an adopted device",
        destructive: true,
        returns: "Value",
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
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Device action name",
            },
        ],
    },
    ActionSpec {
        name: "devices.delete",
        description: "Remove a device from the site",
        destructive: true,
        returns: "void",
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
        name: "clients.action",
        description: "Perform an action on a connected client",
        destructive: true,
        returns: "Value",
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
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Client action name",
            },
        ],
    },
    ActionSpec {
        name: "networks.create",
        description: "Create a new network on a site",
        destructive: true,
        returns: "Network",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "networks.update",
        description: "Update an existing network",
        destructive: true,
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
        name: "networks.delete",
        description: "Delete an existing network",
        destructive: true,
        returns: "void",
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
        name: "wifi.broadcasts.create",
        description: "Create a new WiFi broadcast",
        destructive: true,
        returns: "WifiBroadcast",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "wifi.broadcasts.update",
        description: "Update an existing WiFi broadcast",
        destructive: true,
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
                description: "WiFi broadcast UUID",
            },
        ],
    },
    ActionSpec {
        name: "wifi.broadcasts.delete",
        description: "Delete a WiFi broadcast",
        destructive: true,
        returns: "void",
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
                description: "WiFi broadcast UUID",
            },
        ],
    },
    ActionSpec {
        name: "hotspot.vouchers.list",
        description: "List hotspot vouchers",
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
        name: "hotspot.vouchers.create",
        description: "Generate hotspot vouchers",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "hotspot.vouchers.delete",
        description: "Delete hotspot vouchers by filter",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "filter",
                ty: "string",
                required: true,
                description: "Filter expression",
            },
        ],
    },
    ActionSpec {
        name: "hotspot.vouchers.get",
        description: "Get a hotspot voucher",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "voucher_id",
                ty: "string",
                required: true,
                description: "Voucher UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.zones.list",
        description: "List firewall zones",
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
        name: "firewall.zones.get",
        description: "Get a firewall zone",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.zones.create",
        description: "Create a firewall zone",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.zones.update",
        description: "Update a firewall zone",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.zones.delete",
        description: "Delete a firewall zone",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.list",
        description: "List firewall policies",
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
        name: "firewall.policies.get",
        description: "Get a firewall policy",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.create",
        description: "Create a firewall policy",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.policies.update",
        description: "Update a firewall policy",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.patch",
        description: "Patch a firewall policy",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.ordering.get",
        description: "Get firewall policy ordering",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.policies.ordering.set",
        description: "Set firewall policy ordering",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "acl.rules.list",
        description: "List ACL rules",
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
        name: "acl.rules.get",
        description: "Get an ACL rule",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.create",
        description: "Create an ACL rule",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "acl.rules.update",
        description: "Update an ACL rule",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.delete",
        description: "Delete an ACL rule",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.ordering.get",
        description: "Get ACL rule ordering",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "acl.rules.ordering.set",
        description: "Set ACL rule ordering",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "switching.switch-stacks.list",
        description: "List switch stacks",
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
        name: "switching.switch-stacks.get",
        description: "Get a switch stack",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "switch_stack_id",
                ty: "string",
                required: true,
                description: "Switch stack UUID",
            },
        ],
    },
    ActionSpec {
        name: "switching.mc-lag-domains.list",
        description: "List MC-LAG domains",
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
        name: "switching.mc-lag-domains.get",
        description: "Get an MC-LAG domain",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "mc_lag_domain_id",
                ty: "string",
                required: true,
                description: "MC-LAG domain UUID",
            },
        ],
    },
    ActionSpec {
        name: "switching.lags.list",
        description: "List LAGs",
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
        name: "switching.lags.get",
        description: "Get a LAG",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "lag_id",
                ty: "string",
                required: true,
                description: "LAG UUID",
            },
        ],
    },
    ActionSpec {
        name: "dns.policies.list",
        description: "List DNS policies",
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
        name: "dns.policies.get",
        description: "Get a DNS policy",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "dns.policies.create",
        description: "Create a DNS policy",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "dns.policies.update",
        description: "Update a DNS policy",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "dns.policies.delete",
        description: "Delete a DNS policy",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "traffic-matching-lists.list",
        description: "List traffic matching lists",
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
        name: "traffic-matching-lists.get",
        description: "Get a traffic matching list",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
            },
        ],
    },
    ActionSpec {
        name: "traffic-matching-lists.create",
        description: "Create a traffic matching list",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "traffic-matching-lists.update",
        description: "Update a traffic matching list",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
            },
        ],
    },
    ActionSpec {
        name: "traffic-matching-lists.delete",
        description: "Delete a traffic matching list",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
            },
        ],
    },
    ActionSpec {
        name: "wans.list",
        description: "List WANs",
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
        name: "vpn.site-to-site-tunnels.list",
        description: "List site-to-site VPN tunnels",
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
        name: "vpn.servers.list",
        description: "List VPN servers",
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
        name: "radius.profiles.list",
        description: "List RADIUS profiles",
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
        name: "device-tags.list",
        description: "List device tags",
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
        name: "dpi.categories.list",
        description: "List DPI application categories",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "dpi.applications.list",
        description: "List DPI applications",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "countries.list",
        description: "List countries",
        destructive: false,
        returns: "Page",
        params: &[],
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

fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

fn require_u32(params: &Value, key: &str) -> Result<u32, ToolError> {
    let v = require_i64(params, key)?;
    u32::try_from(v).map_err(|_| ToolError::InvalidParam {
        message: format!("parameter `{key}` must be a non-negative integer"),
        param: key.to_string(),
    })
}

fn object_without(params: &Value, excluded: &[&str]) -> Result<Value, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let filtered: Map<String, Value> = obj
        .iter()
        .filter(|(k, _)| !excluded.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    Ok(Value::Object(filtered))
}

fn query_from(params: &Value, keys: &[&str]) -> Result<Vec<(String, String)>, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let mut out = Vec::new();
    for key in keys {
        if let Some(v) = obj.get(*key) {
            let rendered = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            out.push(((*key).to_string(), rendered));
        }
    }
    Ok(out)
}

/// Dispatch one MCP call against the `UniFi` tool.
#[allow(clippy::too_many_lines)]
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
        "devices.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/devices"), &body)
                .await?;
            to_json(result)
        }
        "devices.port-action" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let port_idx = require_i64(&params, "port_idx")?;
            let body = object_without(&params, &["site_id", "device_id", "port_idx"])?;
            let result = require_client()?
                .post_value(
                    &format!(
                        "/sites/{site_id}/devices/{device_id}/interfaces/ports/{port_idx}/actions"
                    ),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "devices.action" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let body = object_without(&params, &["site_id", "device_id"])?;
            let result = require_client()?
                .post_value(
                    &format!("/sites/{site_id}/devices/{device_id}/actions"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "devices.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/devices/{device_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "clients.action" => {
            let site_id = require_str(&params, "site_id")?;
            let client_id = require_str(&params, "client_id")?;
            let body = object_without(&params, &["site_id", "client_id"])?;
            let result = require_client()?
                .post_value(
                    &format!("/sites/{site_id}/clients/{client_id}/actions"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "networks.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/networks"), &body)
                .await?;
            to_json(result)
        }
        "networks.update" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let body = object_without(&params, &["site_id", "network_id"])?;
            let result = require_client()?
                .put_value(&format!("/sites/{site_id}/networks/{network_id}"), &body)
                .await?;
            to_json(result)
        }
        "networks.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/networks/{network_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "networks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let networks = if q.is_empty() {
                require_client()?.networks_list(site_id).await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/networks"), &q)
                    .await?
            };
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
        "wifi.broadcasts.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/wifi/broadcasts"), &body)
                .await?;
            to_json(result)
        }
        "wifi.broadcasts.update" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            let body = object_without(&params, &["site_id", "wifi_broadcast_id"])?;
            let result = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "wifi.broadcasts.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            require_client()?
                .delete_value(&format!(
                    "/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "hotspot.vouchers.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let vouchers = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/hotspot/vouchers"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/hotspot/vouchers"), &q)
                    .await?
            };
            to_json(vouchers)
        }
        "hotspot.vouchers.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/hotspot/vouchers"), &body)
                .await?;
            to_json(result)
        }
        "hotspot.vouchers.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["filter"])?;
            require_client()?
                .delete_value_query(&format!("/sites/{site_id}/hotspot/vouchers"), &q)
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "hotspot.vouchers.get" => {
            let site_id = require_str(&params, "site_id")?;
            let voucher_id = require_str(&params, "voucher_id")?;
            let voucher = require_client()?
                .get_value(&format!("/sites/{site_id}/hotspot/vouchers/{voucher_id}"))
                .await?;
            to_json(voucher)
        }
        "firewall.zones.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let zones = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/firewall/zones"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/firewall/zones"), &q)
                    .await?
            };
            to_json(zones)
        }
        "firewall.zones.get" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            let zone = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/firewall/zones/{firewall_zone_id}"
                ))
                .await?;
            to_json(zone)
        }
        "firewall.zones.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let zone = require_client()?
                .post_value(&format!("/sites/{site_id}/firewall/zones"), &body)
                .await?;
            to_json(zone)
        }
        "firewall.zones.update" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            let body = object_without(&params, &["site_id", "firewall_zone_id"])?;
            let zone = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/firewall/zones/{firewall_zone_id}"),
                    &body,
                )
                .await?;
            to_json(zone)
        }
        "firewall.zones.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            require_client()?
                .delete_value(&format!(
                    "/sites/{site_id}/firewall/zones/{firewall_zone_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "firewall.policies.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let policies = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/firewall/policies"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/firewall/policies"), &q)
                    .await?
            };
            to_json(policies)
        }
        "firewall.policies.get" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let policy = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/firewall/policies/{firewall_policy_id}"
                ))
                .await?;
            to_json(policy)
        }
        "firewall.policies.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let policy = require_client()?
                .post_value(&format!("/sites/{site_id}/firewall/policies"), &body)
                .await?;
            to_json(policy)
        }
        "firewall.policies.update" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let body = object_without(&params, &["site_id", "firewall_policy_id"])?;
            let policy = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/firewall/policies/{firewall_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "firewall.policies.patch" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let body = object_without(&params, &["site_id", "firewall_policy_id"])?;
            let policy = require_client()?
                .patch_value(
                    &format!("/sites/{site_id}/firewall/policies/{firewall_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "firewall.policies.ordering.get" => {
            let site_id = require_str(&params, "site_id")?;
            let ordering = require_client()?
                .get_value(&format!("/sites/{site_id}/firewall/policies/ordering"))
                .await?;
            to_json(ordering)
        }
        "firewall.policies.ordering.set" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let ordering = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/firewall/policies/ordering"),
                    &body,
                )
                .await?;
            to_json(ordering)
        }
        "acl.rules.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let rules = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/acl-rules"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/acl-rules"), &q)
                    .await?
            };
            to_json(rules)
        }
        "acl.rules.get" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            let rule = require_client()?
                .get_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"))
                .await?;
            to_json(rule)
        }
        "acl.rules.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let rule = require_client()?
                .post_value(&format!("/sites/{site_id}/acl-rules"), &body)
                .await?;
            to_json(rule)
        }
        "acl.rules.update" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            let body = object_without(&params, &["site_id", "acl_rule_id"])?;
            let rule = require_client()?
                .put_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"), &body)
                .await?;
            to_json(rule)
        }
        "acl.rules.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "acl.rules.ordering.get" => {
            let site_id = require_str(&params, "site_id")?;
            let ordering = require_client()?
                .get_value(&format!("/sites/{site_id}/acl-rules/ordering"))
                .await?;
            to_json(ordering)
        }
        "acl.rules.ordering.set" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let ordering = require_client()?
                .put_value(&format!("/sites/{site_id}/acl-rules/ordering"), &body)
                .await?;
            to_json(ordering)
        }
        "switching.switch-stacks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let stacks = require_client()?
                .get_value(&format!("/sites/{site_id}/switching/switch-stacks"))
                .await?;
            to_json(stacks)
        }
        "switching.switch-stacks.get" => {
            let site_id = require_str(&params, "site_id")?;
            let switch_stack_id = require_str(&params, "switch_stack_id")?;
            let stack = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/switching/switch-stacks/{switch_stack_id}"
                ))
                .await?;
            to_json(stack)
        }
        "switching.mc-lag-domains.list" => {
            let site_id = require_str(&params, "site_id")?;
            let domains = require_client()?
                .get_value(&format!("/sites/{site_id}/switching/mc-lag-domains"))
                .await?;
            to_json(domains)
        }
        "switching.mc-lag-domains.get" => {
            let site_id = require_str(&params, "site_id")?;
            let mc_lag_domain_id = require_str(&params, "mc_lag_domain_id")?;
            let domain = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/switching/mc-lag-domains/{mc_lag_domain_id}"
                ))
                .await?;
            to_json(domain)
        }
        "switching.lags.list" => {
            let site_id = require_str(&params, "site_id")?;
            let lags = require_client()?
                .get_value(&format!("/sites/{site_id}/switching/lags"))
                .await?;
            to_json(lags)
        }
        "switching.lags.get" => {
            let site_id = require_str(&params, "site_id")?;
            let lag_id = require_str(&params, "lag_id")?;
            let lag = require_client()?
                .get_value(&format!("/sites/{site_id}/switching/lags/{lag_id}"))
                .await?;
            to_json(lag)
        }
        "dns.policies.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let policies = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/dns/policies"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/dns/policies"), &q)
                    .await?
            };
            to_json(policies)
        }
        "dns.policies.get" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            let policy = require_client()?
                .get_value(&format!("/sites/{site_id}/dns/policies/{dns_policy_id}"))
                .await?;
            to_json(policy)
        }
        "dns.policies.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let policy = require_client()?
                .post_value(&format!("/sites/{site_id}/dns/policies"), &body)
                .await?;
            to_json(policy)
        }
        "dns.policies.update" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            let body = object_without(&params, &["site_id", "dns_policy_id"])?;
            let policy = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/dns/policies/{dns_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "dns.policies.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/dns/policies/{dns_policy_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "traffic-matching-lists.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let lists = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/traffic-matching-lists"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/traffic-matching-lists"), &q)
                    .await?
            };
            to_json(lists)
        }
        "traffic-matching-lists.get" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            let list = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"
                ))
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let list = require_client()?
                .post_value(&format!("/sites/{site_id}/traffic-matching-lists"), &body)
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.update" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            let body = object_without(&params, &["site_id", "traffic_matching_list_id"])?;
            let list = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"),
                    &body,
                )
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            require_client()?
                .delete_value(&format!(
                    "/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "wans.list" => {
            let site_id = require_str(&params, "site_id")?;
            let wans = require_client()?
                .get_value(&format!("/sites/{site_id}/wans"))
                .await?;
            to_json(wans)
        }
        "vpn.site-to-site-tunnels.list" => {
            let site_id = require_str(&params, "site_id")?;
            let tunnels = require_client()?
                .get_value(&format!("/sites/{site_id}/vpn/site-to-site-tunnels"))
                .await?;
            to_json(tunnels)
        }
        "vpn.servers.list" => {
            let site_id = require_str(&params, "site_id")?;
            let servers = require_client()?
                .get_value(&format!("/sites/{site_id}/vpn/servers"))
                .await?;
            to_json(servers)
        }
        "radius.profiles.list" => {
            let site_id = require_str(&params, "site_id")?;
            let profiles = require_client()?
                .get_value(&format!("/sites/{site_id}/radius/profiles"))
                .await?;
            to_json(profiles)
        }
        "device-tags.list" => {
            let site_id = require_str(&params, "site_id")?;
            let tags = require_client()?
                .get_value(&format!("/sites/{site_id}/device-tags"))
                .await?;
            to_json(tags)
        }
        "dpi.categories.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let categories = if q.is_empty() {
                require_client()?.get_value("/dpi/categories").await?
            } else {
                require_client()?
                    .get_value_query("/dpi/categories", &q)
                    .await?
            };
            to_json(categories)
        }
        "dpi.applications.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let applications = if q.is_empty() {
                require_client()?.get_value("/dpi/applications").await?
            } else {
                require_client()?
                    .get_value_query("/dpi/applications", &q)
                    .await?
            };
            to_json(applications)
        }
        "countries.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let countries = if q.is_empty() {
                require_client()?.get_value("/countries").await?
            } else {
                require_client()?.get_value_query("/countries", &q).await?
            };
            to_json(countries)
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
