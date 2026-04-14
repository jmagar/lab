//! Action catalog for the `unraid` service.
//!
//! Single authoritative source for MCP, CLI, and API surfaces.

use lab_apis::core::action::{ActionSpec, ParamSpec};

/// All actions exposed by the `unraid` tool.
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
        description: "Return detailed system information (OS, CPU, versions)",
        destructive: false,
        returns: "SystemInfo",
        params: &[],
    },
    ActionSpec {
        name: "system.metrics",
        description: "Return current CPU and memory utilisation metrics",
        destructive: false,
        returns: "SystemMetrics",
        params: &[],
    },
    ActionSpec {
        name: "system.array",
        description: "Return array state and disk list (data, parity, cache)",
        destructive: false,
        returns: "ArrayStatus",
        params: &[],
    },
    ActionSpec {
        name: "system.online",
        description: "Return whether the Unraid server reports itself as online",
        destructive: false,
        returns: "object: { online: bool }",
        params: &[],
    },
    ActionSpec {
        name: "docker.list",
        description: "List all Docker containers",
        destructive: false,
        returns: "Vec<DockerContainer>",
        params: &[],
    },
    ActionSpec {
        name: "docker.start",
        description: "Start a Docker container",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID (e.g. docker_container:abc123)",
        }],
    },
    ActionSpec {
        name: "docker.stop",
        description: "Stop a Docker container",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID",
        }],
    },
    ActionSpec {
        name: "docker.restart",
        description: "Restart a Docker container (stop then start)",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID",
        }],
    },
    ActionSpec {
        name: "disk.list",
        description: "List physical disks attached to the server",
        destructive: false,
        returns: "Vec<DiskInfo>",
        params: &[],
    },
    // ---- VM actions --------------------------------------------------------
    ActionSpec {
        name: "vm.list",
        description: "List all virtual machines",
        destructive: false,
        returns: "Vec<VmInfo>",
        params: &[],
    },
    ActionSpec {
        name: "vm.start",
        description: "Start a virtual machine",
        destructive: true,
        returns: "{ ok: true, id: string }",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "VM identifier",
        }],
    },
    ActionSpec {
        name: "vm.stop",
        description: "Stop a virtual machine (may corrupt unsaved VM work)",
        destructive: true,
        returns: "{ ok: true, id: string }",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "VM identifier",
        }],
    },
    ActionSpec {
        name: "vm.pause",
        description: "Pause a running virtual machine",
        destructive: true,
        returns: "{ ok: true, id: string }",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "VM identifier",
        }],
    },
    ActionSpec {
        name: "vm.resume",
        description: "Resume a paused virtual machine",
        destructive: false,
        returns: "{ ok: true, id: string }",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "VM identifier",
        }],
    },
    // ---- Notification actions ----------------------------------------------
    ActionSpec {
        name: "notification.list",
        description: "List all server notifications",
        destructive: false,
        returns: "Vec<NotificationInfo>",
        params: &[],
    },
    ActionSpec {
        name: "notification.create",
        description: "Create a new server notification",
        destructive: false,
        returns: "{ ok: true }",
        params: &[
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Notification title",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "Notification body text",
            },
            ParamSpec {
                name: "importance",
                ty: "string",
                required: false,
                description: "Importance level: ALERT | WARNING | NOTICE | INFO",
            },
            ParamSpec {
                name: "type",
                ty: "string",
                required: false,
                description: "Notification type category",
            },
        ],
    },
    ActionSpec {
        name: "notification.archive",
        description: "Archive a notification by ID",
        destructive: true,
        returns: "{ ok: true, id: string }",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Notification ID",
        }],
    },
    // ---- Parity actions ----------------------------------------------------
    ActionSpec {
        name: "parity.history",
        description: "Return parity check history",
        destructive: false,
        returns: "Vec<ParityHistoryEntry>",
        params: &[],
    },
    ActionSpec {
        name: "parity.check-start",
        description: "Start a parity check",
        destructive: false,
        returns: "{ ok: true }",
        params: &[ParamSpec {
            name: "correcting",
            ty: "bool",
            required: false,
            description: "If true, correct errors found during the check",
        }],
    },
    ActionSpec {
        name: "parity.check-pause",
        description: "Pause a running parity check",
        destructive: false,
        returns: "{ ok: true }",
        params: &[],
    },
    ActionSpec {
        name: "parity.check-cancel",
        description: "Cancel a running parity check (destructive — invalidates partial parity data)",
        destructive: true,
        returns: "{ ok: true }",
        params: &[],
    },
    // ---- Share / plugin / network / UPS ------------------------------------
    ActionSpec {
        name: "share.list",
        description: "List all user shares",
        destructive: false,
        returns: "Vec<Share>",
        params: &[],
    },
    ActionSpec {
        name: "plugin.list",
        description: "List installed Unraid plugins",
        destructive: false,
        returns: "Vec<Plugin>",
        params: &[],
    },
    ActionSpec {
        name: "network.list",
        description: "Return network interface information",
        destructive: false,
        returns: "NetworkInfo",
        params: &[],
    },
    ActionSpec {
        name: "ups.devices",
        description: "List UPS devices attached to the server",
        destructive: false,
        returns: "Vec<UpsDevice>",
        params: &[],
    },
    ActionSpec {
        name: "ups.config",
        description: "Return UPS configuration",
        destructive: false,
        returns: "UpsConfig",
        params: &[],
    },
    // ---- Log file ----------------------------------------------------------
    ActionSpec {
        name: "log.read",
        description: "Read lines from a log file on the server",
        destructive: false,
        returns: "{ content: string }",
        params: &[
            ParamSpec {
                name: "path",
                ty: "string",
                required: true,
                description: "Absolute path to the log file (e.g. /var/log/syslog)",
            },
            ParamSpec {
                name: "lines",
                ty: "integer",
                required: false,
                description: "Number of lines to return from the end of the file (default: 50)",
            },
        ],
    },
    // ---- Flash operations --------------------------------------------------
    ActionSpec {
        name: "flash.status",
        description: "Return flash drive status",
        destructive: false,
        returns: "FlashStatus",
        params: &[],
    },
    ActionSpec {
        name: "flash.backup",
        description: "Initiate a flash drive backup (overwrites existing backup)",
        destructive: true,
        returns: "{ ok: true }",
        params: &[],
    },
];
