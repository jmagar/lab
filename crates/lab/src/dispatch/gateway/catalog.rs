use lab_apis::core::action::{ActionSpec, ParamSpec};

const NAME_PARAM: ParamSpec = ParamSpec {
    name: "name",
    ty: "string",
    required: true,
    description: "Gateway name",
};

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
        name: "gateway.list",
        description: "List configured gateways",
        destructive: false,
        returns: "ServerView[]",
        params: &[],
    },
    ActionSpec {
        name: "gateway.supported_services",
        description: "List metadata-backed Lab services that can be added as virtual servers",
        destructive: false,
        returns: "SupportedServiceView[]",
        params: &[],
    },
    ActionSpec {
        name: "gateway.get",
        description: "Get one configured gateway",
        destructive: false,
        returns: "GatewayView",
        params: &[NAME_PARAM],
    },
    ActionSpec {
        name: "gateway.test",
        description: "Test a configured or proposed gateway without saving it",
        destructive: false,
        returns: "GatewayTestResult",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: false,
                description: "Configured gateway name to test",
            },
            ParamSpec {
                name: "spec",
                ty: "json",
                required: false,
                description: "Proposed gateway config payload to test without saving",
            },
        ],
    },
    ActionSpec {
        name: "gateway.add",
        description: "Add a gateway and reconcile runtime state",
        destructive: false,
        returns: "GatewayView",
        params: &[ParamSpec {
            name: "spec",
            ty: "json",
            required: true,
            description: "Gateway config payload to persist",
        }],
    },
    ActionSpec {
        name: "gateway.update",
        description: "Update a gateway and reconcile runtime state",
        destructive: false,
        returns: "GatewayView",
        params: &[
            NAME_PARAM,
            ParamSpec {
                name: "patch",
                ty: "json",
                required: true,
                description: "Partial gateway update payload",
            },
        ],
    },
    ActionSpec {
        name: "gateway.remove",
        description: "Remove a gateway and reconcile runtime state",
        destructive: false,
        returns: "GatewayView",
        params: &[NAME_PARAM],
    },
    ActionSpec {
        name: "gateway.reload",
        description: "Reload gateways from config and reconcile runtime state",
        destructive: false,
        returns: "GatewayCatalogDiff",
        params: &[],
    },
    ActionSpec {
        name: "gateway.status",
        description: "Get current runtime gateway status",
        destructive: false,
        returns: "GatewayRuntimeView[]",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: false,
            description: "Optional gateway name filter",
        }],
    },
    ActionSpec {
        name: "gateway.discovered_tools",
        description: "List discovered upstream tools for one gateway",
        destructive: false,
        returns: "string[]",
        params: &[NAME_PARAM],
    },
    ActionSpec {
        name: "gateway.discovered_resources",
        description: "List discovered upstream resources for one gateway",
        destructive: false,
        returns: "string[]",
        params: &[NAME_PARAM],
    },
    ActionSpec {
        name: "gateway.discovered_prompts",
        description: "List discovered upstream prompts for one gateway",
        destructive: false,
        returns: "string[]",
        params: &[NAME_PARAM],
    },
];
