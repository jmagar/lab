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
    ActionSpec {
        name: "dns.split-get",
        description: "Get the split DNS configuration for the tailnet",
        destructive: false,
        returns: "object",
        params: &[],
    },
    ActionSpec {
        name: "dns.split-set",
        description: "Replace the split DNS configuration for the tailnet",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "config",
            ty: "object",
            required: true,
            description: "Split DNS config object mapping domain suffixes to resolver lists",
        }],
    },
    // ── ACL / Policy ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "acl.get",
        description: "Get the current ACL policy file for the tailnet",
        destructive: false,
        returns: "object",
        params: &[],
    },
    ActionSpec {
        name: "acl.validate",
        description: "Validate an ACL policy file without applying it",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "policy",
            ty: "object",
            required: true,
            description: "HuJSON policy object to validate",
        }],
    },
    ActionSpec {
        name: "acl.set",
        description: "Set the ACL policy file for the tailnet (validates first)",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "policy",
            ty: "object",
            required: true,
            description: "HuJSON policy object to apply",
        }],
    },
    // ── Device extended ops ────────────────────────────────────────────────────
    ActionSpec {
        name: "device.routes-get",
        description: "Get advertised and accepted routes for a device",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "device_id",
            ty: "string",
            required: true,
            description: "Device node ID (nodeId) or legacy numeric ID",
        }],
    },
    ActionSpec {
        name: "device.routes-set",
        description: "Set the subnet routes for a device",
        destructive: false,
        returns: "object",
        params: &[
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device node ID (nodeId) or legacy numeric ID",
            },
            ParamSpec {
                name: "routes",
                ty: "array",
                required: true,
                description: "List of CIDR routes to advertise",
            },
        ],
    },
    ActionSpec {
        name: "device.tag",
        description: "Set tags on a device (replaces existing tags)",
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
                name: "tags",
                ty: "array",
                required: true,
                description: "List of ACL tags to apply (e.g. [\"tag:server\"])",
            },
        ],
    },
    ActionSpec {
        name: "device.expire",
        description: "Expire the key for a device, forcing re-authentication",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "device_id",
            ty: "string",
            required: true,
            description: "Device node ID (nodeId) or legacy numeric ID",
        }],
    },
    // ── Users ──────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.list",
        description: "List all users in the tailnet",
        destructive: false,
        returns: "object",
        params: &[],
    },
    // ── Tailnet Settings ───────────────────────────────────────────────────────
    ActionSpec {
        name: "tailnet.settings-get",
        description: "Get tailnet-level settings",
        destructive: false,
        returns: "object",
        params: &[],
    },
    ActionSpec {
        name: "tailnet.settings-patch",
        description: "Patch tailnet-level settings",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "settings",
            ty: "object",
            required: true,
            description: "Partial settings object with fields to update",
        }],
    },
    // ── Key (extended) ─────────────────────────────────────────────────────────
    ActionSpec {
        name: "key.create",
        description: "Create a new auth key for the tailnet",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "capabilities",
            ty: "object",
            required: true,
            description: "Capabilities object (e.g. {\"devices\":{\"create\":{\"reusable\":true}}})",
        }],
    },
];
