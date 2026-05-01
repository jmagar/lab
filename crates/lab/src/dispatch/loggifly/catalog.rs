use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `loggifly` service.
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
        name: "contract.status",
        description: "Return LoggiFly implementation deferral status",
        destructive: false,
        returns: "ContractStatus",
        params: &[],
    },
];
