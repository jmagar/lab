use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `scrutiny` service.
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
        description: "Probe Scrutiny health endpoint",
        destructive: false,
        returns: "null",
        params: &[],
    },
    ActionSpec {
        name: "dashboard.summary",
        description: "Fetch redacted Scrutiny dashboard summary",
        destructive: false,
        returns: "ScrutinyResponse",
        params: &[],
    },
    ActionSpec {
        name: "device.list",
        description: "Fetch redacted Scrutiny device list",
        destructive: false,
        returns: "ScrutinyResponse",
        params: &[],
    },
];
