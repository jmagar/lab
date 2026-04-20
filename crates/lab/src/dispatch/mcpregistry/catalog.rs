use lab_apis::core::action::{ActionSpec, ParamSpec};

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
        name: "server.list",
        description: "List MCP servers from the registry with optional search and pagination",
        destructive: false,
        returns: "ServerListResponse",
        params: &[
            ParamSpec {
                name: "search",
                ty: "string",
                required: false,
                description: "Search query to filter servers by name or description",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum number of results to return (default: 10, max: 100)",
            },
            ParamSpec {
                name: "cursor",
                ty: "string",
                required: false,
                description: "Pagination cursor from a previous response metadata.nextCursor field",
            },
            ParamSpec {
                name: "version",
                ty: "string",
                required: false,
                description: "Filter by package version string",
            },
            ParamSpec {
                name: "updated_since",
                ty: "string",
                required: false,
                description: "ISO 8601 datetime; return only servers updated after this time",
            },
        ],
    },
    ActionSpec {
        name: "server.get",
        description: "Get details for a single MCP server by its registry name",
        destructive: false,
        returns: "ServerResponse",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Server name as listed in the registry (e.g. `@modelcontextprotocol/server-github`)",
        }],
    },
    ActionSpec {
        name: "server.versions",
        description: "List available versions for a named MCP server",
        destructive: false,
        returns: "ServerListResponse",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Server name to list versions for",
        }],
    },
    ActionSpec {
        name: "server.validate",
        description: "Validate a ServerJSON document against the registry schema without publishing",
        destructive: false,
        returns: "ValidationResult",
        params: &[ParamSpec {
            name: "server_json",
            ty: "object",
            required: true,
            description: "ServerJSON document to validate (must include name, description, version)",
        }],
    },
];
