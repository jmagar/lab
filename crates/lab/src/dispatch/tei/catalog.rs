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
        description: "Check whether the TEI server is healthy",
        destructive: false,
        returns: "void",
        params: &[],
    },
    ActionSpec {
        name: "server.info",
        description: "Get served model and runtime metadata",
        destructive: false,
        returns: "Info",
        params: &[],
    },
    ActionSpec {
        name: "embed.create",
        description: "Generate embeddings for one or more input strings",
        destructive: false,
        returns: "number[][]",
        params: &[
            ParamSpec {
                name: "input",
                ty: "string",
                required: false,
                description: "Single input string shortcut",
            },
            ParamSpec {
                name: "inputs",
                ty: "json",
                required: false,
                description: "Array of input strings",
            },
            ParamSpec {
                name: "normalize",
                ty: "bool",
                required: false,
                description: "Whether to normalize the returned vectors",
            },
            ParamSpec {
                name: "truncate",
                ty: "bool",
                required: false,
                description: "Whether to truncate overlong inputs",
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
