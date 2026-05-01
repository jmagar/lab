use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `glances` service.
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
        name: "server.health",
        description: "Probe Glances API health",
        destructive: false,
        returns: "null",
        params: &[],
    },
    ActionSpec {
        name: "system.info",
        description: "Fetch Glances system info",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "quicklook.summary",
        description: "Fetch Glances quicklook summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "cpu.summary",
        description: "Fetch Glances CPU summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "memory.summary",
        description: "Fetch Glances memory summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "load.summary",
        description: "Fetch Glances load summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "network.summary",
        description: "Fetch Glances network summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "diskio.summary",
        description: "Fetch Glances disk I/O summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "fs.summary",
        description: "Fetch Glances filesystem summary",
        destructive: false,
        returns: "GlancesResponse",
        params: &[],
    },
    ActionSpec {
        name: "process.top",
        description: "Fetch top Glances processes with process args redacted",
        destructive: false,
        returns: "GlancesResponse",
        params: &[ParamSpec {
            name: "limit",
            ty: "integer",
            required: true,
            description: "Maximum processes, 1-50",
        }],
    },
];
