use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Single authoritative action catalog for the `Tailscale` service.
///
/// MCP, CLI, and API all reference this — never copy the array.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        returns: "Schema",
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
    },
    // ── Devices ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "device.list",
        description: "List all devices in the tailnet",
        destructive: false,
        returns: "DeviceList",
        params: &[],
    },
    ActionSpec {
        name: "device.get",
        description: "Get details for a specific device by ID",
        destructive: false,
        returns: "Device",
        params: &[ParamSpec {
            name: "device_id",
            ty: "string",
            required: true,
            description: "Device node ID (nodeId) or legacy numeric ID",
        }],
    },
    ActionSpec {
        name: "device.delete",
        description: "Remove a device from the tailnet",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "device_id",
            ty: "string",
            required: true,
            description: "Device node ID (nodeId) or legacy numeric ID",
        }],
    },
    ActionSpec {
        name: "device.authorize",
        description: "Authorize or de-authorize a device",
        destructive: false,
        returns: "void",
        params: &[
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device node ID (nodeId) or legacy numeric ID",
            },
            ParamSpec {
                name: "authorized",
                ty: "bool",
                required: true,
                description: "true to authorize, false to de-authorize",
            },
        ],
    },
    // ── Auth Keys ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "key.list",
        description: "List all auth keys for the tailnet",
        destructive: false,
        returns: "KeyList",
        params: &[],
    },
    ActionSpec {
        name: "key.get",
        description: "Get details for a specific auth key",
        destructive: false,
        returns: "AuthKey",
        params: &[ParamSpec {
            name: "key_id",
            ty: "string",
            required: true,
            description: "Auth key ID",
        }],
    },
    ActionSpec {
        name: "key.delete",
        description: "Delete an auth key",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "key_id",
            ty: "string",
            required: true,
            description: "Auth key ID",
        }],
    },
    // ── DNS ───────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "dns.nameservers",
        description: "Get DNS nameservers configured for the tailnet",
        destructive: false,
        returns: "DnsNameservers",
        params: &[],
    },
    ActionSpec {
        name: "dns.search_paths",
        description: "Get DNS search paths configured for the tailnet",
        destructive: false,
        returns: "DnsSearchPaths",
        params: &[],
    },
];
