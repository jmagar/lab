use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `dozzle` service.
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
        description: "Probe Dozzle health",
        destructive: false,
        returns: "HealthResponse",
        params: &[],
    },
    ActionSpec {
        name: "server.version",
        description: "Fetch Dozzle version",
        destructive: false,
        returns: "VersionResponse",
        params: &[],
    },
    ActionSpec {
        name: "containers.list",
        description: "Return the first bounded Dozzle container inventory event",
        destructive: false,
        returns: "ContainersListResponse",
        params: &[
            ParamSpec {
                name: "max_events",
                ty: "integer",
                required: false,
                description: "Maximum SSE events to inspect before stopping",
            },
            ParamSpec {
                name: "max_bytes",
                ty: "integer",
                required: false,
                description: "Maximum bytes to read from the SSE stream",
            },
            ParamSpec {
                name: "timeout_ms",
                ty: "integer",
                required: false,
                description: "Maximum stream read duration in milliseconds",
            },
        ],
    },
    ActionSpec {
        name: "logs.fetch",
        description: "Fetch bounded container logs as parsed JSONL events",
        destructive: false,
        returns: "LogFetchResponse",
        params: &[
            ParamSpec {
                name: "host",
                ty: "string",
                required: true,
                description: "Dozzle host id",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container id",
            },
            ParamSpec {
                name: "stdout",
                ty: "bool",
                required: false,
                description: "Include stdout logs; defaults to true when stderr is absent",
            },
            ParamSpec {
                name: "stderr",
                ty: "bool",
                required: false,
                description: "Include stderr logs; defaults to true when stdout is absent",
            },
            ParamSpec {
                name: "max_lines",
                ty: "integer",
                required: false,
                description: "Maximum JSONL rows to parse",
            },
            ParamSpec {
                name: "max_bytes",
                ty: "integer",
                required: false,
                description: "Maximum bytes to read from Dozzle",
            },
            ParamSpec {
                name: "timeout_ms",
                ty: "integer",
                required: false,
                description: "Maximum read duration in milliseconds",
            },
        ],
    },
    ActionSpec {
        name: "logs.fetch-plain",
        description: "Fetch bounded container logs as plain text",
        destructive: false,
        returns: "PlainLogFetchResponse",
        params: &[
            ParamSpec {
                name: "host",
                ty: "string",
                required: true,
                description: "Dozzle host id",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container id",
            },
            ParamSpec {
                name: "stdout",
                ty: "bool",
                required: false,
                description: "Include stdout logs; defaults to true when stderr is absent",
            },
            ParamSpec {
                name: "stderr",
                ty: "bool",
                required: false,
                description: "Include stderr logs; defaults to true when stdout is absent",
            },
            ParamSpec {
                name: "max_lines",
                ty: "integer",
                required: false,
                description: "Maximum lines reported in metadata",
            },
            ParamSpec {
                name: "max_bytes",
                ty: "integer",
                required: false,
                description: "Maximum bytes to read from Dozzle",
            },
            ParamSpec {
                name: "timeout_ms",
                ty: "integer",
                required: false,
                description: "Maximum read duration in milliseconds",
            },
        ],
    },
];
