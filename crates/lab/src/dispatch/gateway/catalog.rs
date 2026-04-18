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
        name: "gateway.server.get",
        description: "Get one unified server row by id",
        destructive: false,
        returns: "ServerView",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Unified server id",
        }],
    },
    ActionSpec {
        name: "gateway.supported_services",
        description: "List metadata-backed Lab services that can be added as virtual servers",
        destructive: false,
        returns: "SupportedServiceView[]",
        params: &[],
    },
    ActionSpec {
        name: "gateway.virtual_server.enable",
        description: "Enable a configured Lab-backed service as a virtual server",
        destructive: true,
        returns: "ServerView",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Virtual server id",
        }],
    },
    ActionSpec {
        name: "gateway.virtual_server.disable",
        description: "Disable a Lab-backed virtual server without removing its config",
        destructive: true,
        returns: "ServerView",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Virtual server id",
        }],
    },
    ActionSpec {
        name: "gateway.virtual_server.set_surface",
        description: "Enable or disable one surface on a Lab-backed virtual server",
        destructive: true,
        returns: "ServerView",
        params: &[
            ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Virtual server id",
            },
            ParamSpec {
                name: "surface",
                ty: "string",
                required: true,
                description: "Surface name: cli, api, mcp, or webui",
            },
            ParamSpec {
                name: "enabled",
                ty: "boolean",
                required: true,
                description: "Whether the surface should be enabled",
            },
        ],
    },
    ActionSpec {
        name: "gateway.virtual_server.get_mcp_policy",
        description: "Read the MCP action allowlist for a Lab-backed virtual server",
        destructive: false,
        returns: "VirtualServerMcpPolicyView",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Virtual server id",
        }],
    },
    ActionSpec {
        name: "gateway.virtual_server.set_mcp_policy",
        description: "Replace the MCP action allowlist for a Lab-backed virtual server",
        destructive: true,
        returns: "VirtualServerMcpPolicyView",
        params: &[
            ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Virtual server id",
            },
            ParamSpec {
                name: "allowed_actions",
                ty: "string[]",
                required: true,
                description: "Exact Lab action names to expose. Empty means expose all actions.",
            },
        ],
    },
    ActionSpec {
        name: "gateway.service_config.get",
        description: "Read canonical config for one Lab-backed service",
        destructive: false,
        returns: "ServiceConfigView",
        params: &[ParamSpec {
            name: "service",
            ty: "string",
            required: true,
            description: "Service key",
        }],
    },
    ActionSpec {
        name: "gateway.service_actions",
        description: "List compiled action metadata for one Lab-backed service",
        destructive: false,
        returns: "ServiceActionView[]",
        params: &[ParamSpec {
            name: "service",
            ty: "string",
            required: true,
            description: "Service key",
        }],
    },
    ActionSpec {
        name: "gateway.service_config.set",
        description: "Write canonical config for one Lab-backed service",
        destructive: true,
        returns: "ServiceConfigView",
        params: &[
            ParamSpec {
                name: "service",
                ty: "string",
                required: true,
                description: "Service key",
            },
            ParamSpec {
                name: "values",
                ty: "json",
                required: true,
                description: "Env-field map to persist for this service",
            },
        ],
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
        destructive: true,
        returns: "GatewayView",
        params: &[
            ParamSpec {
                name: "spec",
                ty: "json",
                required: true,
                description: "Gateway config payload to persist",
            },
            ParamSpec {
                name: "bearer_token_value",
                ty: "string",
                required: false,
                description: "Write-only: raw bearer token to store securely. Never returned on reads. Requires bearer_token_env in spec to name the env var.",
            },
        ],
    },
    ActionSpec {
        name: "gateway.update",
        description: "Update a gateway and reconcile runtime state",
        destructive: true,
        returns: "GatewayView",
        params: &[
            NAME_PARAM,
            ParamSpec {
                name: "patch",
                ty: "json",
                required: true,
                description: "Partial gateway update payload",
            },
            ParamSpec {
                name: "bearer_token_value",
                ty: "string",
                required: false,
                description: "Write-only: raw bearer token to store securely. Never returned on reads. Requires bearer_token_env in patch or existing config.",
            },
        ],
    },
    ActionSpec {
        name: "gateway.remove",
        description: "Remove a gateway and reconcile runtime state",
        destructive: true,
        returns: "GatewayView",
        params: &[NAME_PARAM],
    },
    ActionSpec {
        name: "gateway.reload",
        description: "Reload gateways from config and reconcile runtime state",
        destructive: true,
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
        description: "List discovered upstream tool metadata and exposure state for one gateway",
        destructive: false,
        returns: "GatewayToolExposureRowView[]",
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

#[cfg(test)]
mod tests {
    use super::ACTIONS;

    #[test]
    fn gateway_reconciliation_actions_are_marked_destructive() {
        for action in [
            "gateway.add",
            "gateway.update",
            "gateway.remove",
            "gateway.reload",
        ] {
            let spec = ACTIONS
                .iter()
                .find(|spec| spec.name == action)
                .expect("gateway action");
            assert!(spec.destructive, "{action} should be destructive");
        }
    }

    #[test]
    fn gateway_read_actions_remain_non_destructive() {
        for action in [
            "gateway.list",
            "gateway.get",
            "gateway.test",
            "gateway.status",
            "gateway.discovered_tools",
            "gateway.discovered_resources",
            "gateway.discovered_prompts",
        ] {
            let spec = ACTIONS
                .iter()
                .find(|spec| spec.name == action)
                .expect("gateway action");
            assert!(!spec.destructive, "{action} should remain non-destructive");
        }
    }
}
