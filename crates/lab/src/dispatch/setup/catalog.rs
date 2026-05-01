//! Action catalog for the `setup` Bootstrap orchestrator.

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
        name: "state",
        description: "First-run + draft snapshot for the wizard / settings UI",
        destructive: false,
        returns: "SetupSnapshot",
        params: &[],
    },
    ActionSpec {
        name: "schema.get",
        description: "UiSchema projection for all (or filtered) services",
        destructive: false,
        returns: "ServiceSchemaMap",
        params: &[ParamSpec {
            name: "services",
            ty: "string[]",
            required: false,
            description: "Optional filter; defaults to every service in the registry",
        }],
    },
    ActionSpec {
        name: "draft.get",
        description: "Read .env.draft with secret values masked to '***'",
        destructive: false,
        returns: "DraftEntry[]",
        params: &[],
    },
    ActionSpec {
        name: "draft.set",
        description: "Write a key (or section) into .env.draft (validated server-side)",
        destructive: false,
        returns: "DraftSetOutcome",
        params: &[
            ParamSpec {
                name: "entries",
                ty: "DraftEntry[]",
                required: true,
                description: "Key/value pairs to write into the draft",
            },
            ParamSpec {
                name: "force",
                ty: "bool",
                required: false,
                description: "Overwrite conflicting draft keys (default false)",
            },
        ],
    },
    ActionSpec {
        name: "draft.commit",
        description: "Run audit and atomically merge .env.draft into .env",
        destructive: true,
        returns: "CommitOutcome",
        params: &[ParamSpec {
            name: "force",
            ty: "bool",
            required: false,
            description: "Overwrite conflicting .env keys (default false)",
        }],
    },
    ActionSpec {
        name: "finalize",
        description: "Alias for draft.commit returning a summary envelope",
        destructive: true,
        returns: "CommitOutcome",
        params: &[],
    },
];
