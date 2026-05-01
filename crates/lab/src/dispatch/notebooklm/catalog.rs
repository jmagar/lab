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
        name: "notebook.list",
        description: "List NotebookLM notebooks",
        destructive: false,
        returns: "Notebook[]",
        params: &[],
    },
    ActionSpec {
        name: "notebook.create",
        description: "Create a NotebookLM notebook",
        destructive: false,
        returns: "Notebook",
        params: &[ParamSpec {
            name: "title",
            ty: "string",
            required: true,
            description: "Notebook title",
        }],
    },
    ActionSpec {
        name: "notebook.get",
        description: "Get NotebookLM notebook details",
        destructive: false,
        returns: "Notebook",
        params: &[ParamSpec {
            name: "notebook_id",
            ty: "string",
            required: true,
            description: "Notebook ID",
        }],
    },
    ActionSpec {
        name: "notebook.delete",
        description: "Delete a NotebookLM notebook",
        destructive: true,
        returns: "DeleteResult",
        params: &[ParamSpec {
            name: "notebook_id",
            ty: "string",
            required: true,
            description: "Notebook ID",
        }],
    },
    ActionSpec {
        name: "source.list",
        description: "List sources in a NotebookLM notebook",
        destructive: false,
        returns: "Source[]",
        params: &[ParamSpec {
            name: "notebook_id",
            ty: "string",
            required: true,
            description: "Notebook ID",
        }],
    },
    ActionSpec {
        name: "source.add_url",
        description: "Add a URL source to a NotebookLM notebook",
        destructive: false,
        returns: "Source",
        params: &[
            ParamSpec {
                name: "notebook_id",
                ty: "string",
                required: true,
                description: "Notebook ID",
            },
            ParamSpec {
                name: "url",
                ty: "string",
                required: true,
                description: "URL to add as a source",
            },
        ],
    },
    ActionSpec {
        name: "server.health",
        description: "Check whether NotebookLM is reachable and authenticated",
        destructive: false,
        returns: "Health",
        params: &[],
    },
];
