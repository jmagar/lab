use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `freshrss` service.
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
        name: "subscription.list",
        description: "List FreshRSS subscriptions",
        destructive: false,
        returns: "FreshRssResponse",
        params: &[],
    },
    ActionSpec {
        name: "tag.list",
        description: "List FreshRSS tags",
        destructive: false,
        returns: "FreshRssResponse",
        params: &[],
    },
    ActionSpec {
        name: "unread.counts",
        description: "Fetch FreshRSS unread counts",
        destructive: false,
        returns: "FreshRssResponse",
        params: &[],
    },
    ActionSpec {
        name: "stream.items",
        description: "Fetch one bounded FreshRSS reading-list page",
        destructive: false,
        returns: "FreshRssResponse",
        params: &[
            ParamSpec {
                name: "n",
                ty: "integer",
                required: true,
                description: "Maximum items, 1-100",
            },
            ParamSpec {
                name: "continuation",
                ty: "string",
                required: false,
                description: "Continuation token from a previous response",
            },
        ],
    },
];
