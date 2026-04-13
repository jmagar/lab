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
        name: "server.health",
        description: "Check whether the Apprise API is healthy",
        destructive: false,
        returns: "HealthStatus",
        params: &[],
    },
    ActionSpec {
        name: "notify.send",
        description: "Send a stateless notification to the supplied URLs",
        destructive: false,
        returns: "void",
        params: &[
            ParamSpec {
                name: "body",
                ty: "string",
                required: false,
                description: "Notification body text (optional when `payload` override is supplied)",
            },
            ParamSpec {
                name: "urls",
                ty: "json",
                required: false,
                description: "One URL string or an array of URL strings",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "Optional notification title",
            },
            ParamSpec {
                name: "type",
                ty: "string",
                required: false,
                description: "Message type: info, success, warning, or failure",
            },
            ParamSpec {
                name: "format",
                ty: "string",
                required: false,
                description: "Body format: text, markdown, or html",
            },
            ParamSpec {
                name: "tag",
                ty: "string",
                required: false,
                description: "Optional tag filter",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Full request body override",
            },
        ],
    },
    ActionSpec {
        name: "notify.key.send",
        description: "Send a notification using a stored config key",
        destructive: false,
        returns: "void",
        params: &[
            ParamSpec {
                name: "key",
                ty: "string",
                required: true,
                description: "Stored config key",
            },
            ParamSpec {
                name: "body",
                ty: "string",
                required: false,
                description: "Notification body text (optional when `payload` override is supplied)",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "Optional notification title",
            },
            ParamSpec {
                name: "type",
                ty: "string",
                required: false,
                description: "Message type: info, success, warning, or failure",
            },
            ParamSpec {
                name: "format",
                ty: "string",
                required: false,
                description: "Body format: text, markdown, or html",
            },
            ParamSpec {
                name: "tag",
                ty: "string",
                required: false,
                description: "Optional tag filter",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Full request body override",
            },
        ],
    },
];
