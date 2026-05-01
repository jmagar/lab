use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `pihole` service.
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
        name: "server.summary",
        description: "Fetch Pi-hole summary counters",
        destructive: false,
        returns: "PiholeResponse",
        params: &[],
    },
    ActionSpec {
        name: "server.settings",
        description: "Fetch Pi-hole settings",
        destructive: false,
        returns: "PiholeResponse",
        params: &[],
    },
    ActionSpec {
        name: "blocking.status",
        description: "Fetch DNS blocking status",
        destructive: false,
        returns: "PiholeResponse",
        params: &[],
    },
    ActionSpec {
        name: "blocking.set",
        description: "Enable or disable DNS blocking",
        destructive: true,
        returns: "PiholeResponse",
        params: &[
            ParamSpec {
                name: "blocking",
                ty: "boolean",
                required: true,
                description: "Whether blocking should be enabled",
            },
            ParamSpec {
                name: "timer_seconds",
                ty: "integer",
                required: false,
                description: "Optional duration for temporary enable/disable",
            },
        ],
    },
    ActionSpec {
        name: "querylog.search",
        description: "Search bounded Pi-hole query logs",
        destructive: false,
        returns: "QueryLogResponse",
        params: &[
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Offset, defaults to 0",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: true,
                description: "Maximum rows, 1-500",
            },
        ],
    },
    ActionSpec {
        name: "adlist.list",
        description: "List Pi-hole adlists",
        destructive: false,
        returns: "PiholeResponse",
        params: &[],
    },
    ActionSpec {
        name: "adlist.add",
        description: "Add a Pi-hole adlist",
        destructive: true,
        returns: "PiholeResponse",
        params: &[ParamSpec {
            name: "address",
            ty: "string",
            required: true,
            description: "Adlist URL/address",
        }],
    },
    ActionSpec {
        name: "adlist.remove",
        description: "Remove a Pi-hole adlist by id",
        destructive: true,
        returns: "PiholeResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Adlist id",
        }],
    },
    ActionSpec {
        name: "domain.list",
        description: "List Pi-hole domain rules",
        destructive: false,
        returns: "PiholeResponse",
        params: &[],
    },
    ActionSpec {
        name: "domain.add",
        description: "Add a Pi-hole domain rule",
        destructive: true,
        returns: "PiholeResponse",
        params: &[
            ParamSpec {
                name: "domain",
                ty: "string",
                required: true,
                description: "Domain or regex pattern",
            },
            ParamSpec {
                name: "domain_type",
                ty: "integer",
                required: true,
                description: "0 exact-allow, 1 exact-deny, 2 regex-allow, 3 regex-deny",
            },
            ParamSpec {
                name: "comment",
                ty: "string",
                required: false,
                description: "Optional comment",
            },
        ],
    },
];
