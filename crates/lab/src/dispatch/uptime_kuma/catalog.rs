use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `uptime-kuma` service.
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
    ActionSpec {
        name: "contract.status",
        description: "Show the Uptime Kuma integration contract and live-read status",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "server.health",
        description: "Probe the Uptime Kuma web UI root",
        destructive: false,
        returns: "null",
        params: &[],
    },
    ActionSpec {
        name: "monitor.list",
        description: "List monitors through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "monitor.get",
        description: "Fetch one monitor through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Monitor id",
        }],
    },
    ActionSpec {
        name: "heartbeat.list",
        description: "List heartbeat rows through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "monitor_id",
            ty: "string",
            required: false,
            description: "Optional monitor id filter",
        }],
    },
    ActionSpec {
        name: "status.summary",
        description: "Summarize monitor status through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "notification.list",
        description: "List notification channels through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
];
