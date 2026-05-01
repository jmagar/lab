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
        description: "Return LoggiFly local integration contract status",
        destructive: false,
        returns: "ContractStatus",
        params: &[],
    },
    ActionSpec {
        name: "health.status",
        description: "Inspect the documented LoggiFly heartbeat file health status",
        destructive: false,
        returns: "HealthStatus",
        params: &[],
    },
    ActionSpec {
        name: "config.summary",
        description: "Summarize allowlisted LoggiFly config.yaml without returning raw config",
        destructive: false,
        returns: "ConfigSummary",
        params: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::ACTIONS;

    #[test]
    fn loggifly_local_actions_are_read_only() {
        for action in ["contract.status", "health.status", "config.summary"] {
            let spec = ACTIONS.iter().find(|spec| spec.name == action).unwrap();
            assert!(!spec.destructive, "{action} should remain read-only");
        }
    }
}
