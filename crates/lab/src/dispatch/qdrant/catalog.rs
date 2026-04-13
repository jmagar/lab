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
        description: "Check whether the Qdrant server is healthy",
        destructive: false,
        returns: "void",
        params: &[],
    },
    ActionSpec {
        name: "collections.list",
        description: "List collection names",
        destructive: false,
        returns: "CollectionDescription[]",
        params: &[],
    },
    ActionSpec {
        name: "collections.get",
        description: "Get raw metadata for one collection",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Collection name",
        }],
    },
];
