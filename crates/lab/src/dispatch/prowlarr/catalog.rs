use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Prowlarr` service.
///
/// This is the single authoritative source. MCP, CLI, and API re-export
/// or reference it.
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
        name: "indexers.list",
        description: "List all configured indexers",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "indexers.get",
        description: "Get a single indexer by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Indexer ID",
        }],
    },
    ActionSpec {
        name: "indexers.delete",
        description: "Delete an indexer by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Indexer ID",
        }],
    },
    ActionSpec {
        name: "indexers.test",
        description: "Test a single indexer by ID (fetches the indexer then POSTs to test endpoint)",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Indexer ID",
        }],
    },
    ActionSpec {
        name: "indexers.testall",
        description: "Test all configured indexers",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "indexers.categories",
        description: "List all indexer categories",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "history.list",
        description: "Get history with optional filters",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "integer",
                required: false,
                description: "Number of results per page",
            },
            ParamSpec {
                name: "sort_key",
                ty: "string",
                required: false,
                description: "Field to sort by (e.g. date)",
            },
            ParamSpec {
                name: "sort_dir",
                ty: "string",
                required: false,
                description: "Sort direction: ascending or descending",
            },
            ParamSpec {
                name: "indexer_id",
                ty: "integer",
                required: false,
                description: "Filter history to a specific indexer ID",
            },
        ],
    },
    ActionSpec {
        name: "applications.list",
        description: "List configured applications (download clients connected to Prowlarr)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "applications.get",
        description: "Get a single application by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Application ID",
        }],
    },
    ActionSpec {
        name: "applications.delete",
        description: "Delete an application by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Application ID",
        }],
    },
    ActionSpec {
        name: "system.status",
        description: "Get system status (version, branch, OS, etc.)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "system.health",
        description: "Get system health alerts",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
