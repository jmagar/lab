use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Single authoritative action catalog for the `arcane` service.
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
    // ── Health ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "health",
        description: "Check Arcane API health",
        destructive: false,
        returns: "HealthResponse",
        params: &[],
    },
    // ── Environments ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "environment.list",
        description: "List all registered Docker environments",
        destructive: false,
        returns: "Environment[]",
        params: &[],
    },
    ActionSpec {
        name: "environment.get",
        description: "Get details for a specific environment by ID",
        destructive: false,
        returns: "Environment",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    // ── Containers ────────────────────────────────────────────────────────────
    ActionSpec {
        name: "container.list",
        description: "List all containers in an environment",
        destructive: false,
        returns: "Container[]",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    ActionSpec {
        name: "container.get",
        description: "Get details for a specific container",
        destructive: false,
        returns: "Container",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container ID",
            },
        ],
    },
    ActionSpec {
        name: "container.start",
        description: "Start a stopped container",
        destructive: false,
        returns: "ContainerActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container ID",
            },
        ],
    },
    ActionSpec {
        name: "container.stop",
        description: "Stop a running container",
        destructive: false,
        returns: "ContainerActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container ID",
            },
        ],
    },
    ActionSpec {
        name: "container.restart",
        description: "Restart a container",
        destructive: false,
        returns: "ContainerActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container ID",
            },
        ],
    },
    ActionSpec {
        name: "container.redeploy",
        description: "Redeploy a container (pull latest image and recreate)",
        destructive: true,
        returns: "ContainerActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "container_id",
                ty: "string",
                required: true,
                description: "Container ID",
            },
        ],
    },
];
