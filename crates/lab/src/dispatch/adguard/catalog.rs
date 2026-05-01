use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `adguard` service.
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
        name: "server.status",
        description: "Fetch AdGuard Home status",
        destructive: false,
        returns: "AdguardResponse",
        params: &[],
    },
    ActionSpec {
        name: "server.version",
        description: "Fetch AdGuard Home version",
        destructive: false,
        returns: "AdguardResponse",
        params: &[],
    },
    ActionSpec {
        name: "stats.summary",
        description: "Fetch AdGuard Home stats summary",
        destructive: false,
        returns: "AdguardResponse",
        params: &[],
    },
    ActionSpec {
        name: "querylog.search",
        description: "Search bounded AdGuard Home DNS query log",
        destructive: false,
        returns: "QueryLogResponse",
        params: &[
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: true,
                description: "Maximum rows, 1-200",
            },
            ParamSpec {
                name: "search",
                ty: "string",
                required: false,
                description: "Optional AdGuard query-log search",
            },
            ParamSpec {
                name: "older_than",
                ty: "string",
                required: false,
                description: "Optional upstream cursor timestamp",
            },
        ],
    },
    ActionSpec {
        name: "filtering.status",
        description: "Fetch DNS filtering status",
        destructive: false,
        returns: "AdguardResponse",
        params: &[],
    },
    ActionSpec {
        name: "filtering.check-host",
        description: "Check whether a hostname is filtered",
        destructive: false,
        returns: "AdguardResponse",
        params: &[ParamSpec {
            name: "host",
            ty: "string",
            required: true,
            description: "Hostname to check",
        }],
    },
];
