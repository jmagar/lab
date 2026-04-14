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
    // ── Projects ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "project.list",
        description: "List all Compose/Docker projects in an environment",
        destructive: false,
        returns: "Project[]",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    ActionSpec {
        name: "project.create",
        description: "Create a new project in an environment",
        destructive: false,
        returns: "Project",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "body",
                ty: "object",
                required: true,
                description: "Project creation payload (JSON object)",
            },
        ],
    },
    ActionSpec {
        name: "project.up",
        description: "Bring a project up (docker compose up)",
        destructive: false,
        returns: "ProjectActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "project_id",
                ty: "string",
                required: true,
                description: "Project ID",
            },
        ],
    },
    ActionSpec {
        name: "project.down",
        description: "Bring a project down (docker compose down)",
        destructive: true,
        returns: "ProjectActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "project_id",
                ty: "string",
                required: true,
                description: "Project ID",
            },
        ],
    },
    ActionSpec {
        name: "project.redeploy",
        description: "Redeploy a project (pull latest images and recreate)",
        destructive: false,
        returns: "ProjectActionResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "project_id",
                ty: "string",
                required: true,
                description: "Project ID",
            },
        ],
    },
    // ── Volumes ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "volume.list",
        description: "List all Docker volumes in an environment",
        destructive: false,
        returns: "Volume[]",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    ActionSpec {
        name: "volume.delete",
        description: "Delete a Docker volume by name",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "volume_name",
                ty: "string",
                required: true,
                description: "Volume name to delete",
            },
        ],
    },
    ActionSpec {
        name: "volume.prune",
        description: "Prune all unused Docker volumes in an environment",
        destructive: true,
        returns: "PruneResult",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    // ── Images ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "image.list",
        description: "List all Docker images in an environment",
        destructive: false,
        returns: "Image[]",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    ActionSpec {
        name: "image.pull",
        description: "Pull a Docker image in an environment",
        destructive: false,
        returns: "ImagePullResult",
        params: &[
            ParamSpec {
                name: "env_id",
                ty: "string",
                required: true,
                description: "Environment ID",
            },
            ParamSpec {
                name: "image",
                ty: "string",
                required: true,
                description: "Image reference to pull (e.g. nginx:latest)",
            },
        ],
    },
    ActionSpec {
        name: "image.prune",
        description: "Prune unused Docker images in an environment",
        destructive: true,
        returns: "ImagePruneResult",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
    ActionSpec {
        name: "image.update-summary",
        description: "Get a summary of available image updates in an environment",
        destructive: false,
        returns: "ImageUpdateSummary",
        params: &[ParamSpec {
            name: "env_id",
            ty: "string",
            required: true,
            description: "Environment ID",
        }],
    },
];
