use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `immich` service.
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
        description: "Probe Immich server reachability",
        destructive: false,
        returns: "null",
        params: &[],
    },
    ActionSpec {
        name: "server.info",
        description: "Fetch allowlisted Immich server info",
        destructive: false,
        returns: "ServerInfo",
        params: &[],
    },
    ActionSpec {
        name: "server.version",
        description: "Fetch Immich server version",
        destructive: false,
        returns: "ServerInfo",
        params: &[],
    },
    ActionSpec {
        name: "user.me",
        description: "Fetch the authenticated Immich user summary",
        destructive: false,
        returns: "UserMe",
        params: &[],
    },
    ActionSpec {
        name: "asset.search",
        description: "Search Immich assets with a hard result cap",
        destructive: false,
        returns: "AssetSearchResponse",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: false,
                description: "Optional search query",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: true,
                description: "Maximum assets to return, 1-50",
            },
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Optional upstream page",
            },
        ],
    },
    ActionSpec {
        name: "asset.get",
        description: "Fetch redacted metadata for one Immich asset",
        destructive: false,
        returns: "AssetMetadata",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Immich asset id",
        }],
    },
];
