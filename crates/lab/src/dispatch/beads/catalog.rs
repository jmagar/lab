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
        name: "contract.status",
        description: "Return Beads local CLI integration contract status",
        destructive: false,
        returns: "ContractStatus",
        params: &[],
    },
    ActionSpec {
        name: "health.status",
        description: "Check bd availability and workspace status",
        destructive: false,
        returns: "BeadsHealth",
        params: &[],
    },
    ActionSpec {
        name: "version.get",
        description: "Return bd version metadata",
        destructive: false,
        returns: "BdJson",
        params: &[],
    },
    ActionSpec {
        name: "context.get",
        description: "Return active Beads workspace/backend context",
        destructive: false,
        returns: "BdJson",
        params: &[],
    },
    ActionSpec {
        name: "status.summary",
        description: "Return Beads database summary counts",
        destructive: false,
        returns: "BdJson",
        params: &[],
    },
    ActionSpec {
        name: "issue.list",
        description: "List Beads issues with optional status and limit filters",
        destructive: false,
        returns: "BdJson",
        params: &[
            ParamSpec {
                name: "status",
                ty: "string",
                required: false,
                description: "Optional stored status filter: open, in_progress, blocked, deferred, closed",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum issues to return, capped at 500",
            },
        ],
    },
    ActionSpec {
        name: "issue.ready",
        description: "List ready unblocked Beads issues",
        destructive: false,
        returns: "BdJson",
        params: &[ParamSpec {
            name: "limit",
            ty: "integer",
            required: false,
            description: "Maximum issues to return, capped at 500",
        }],
    },
    ActionSpec {
        name: "issue.show",
        description: "Show one Beads issue by id",
        destructive: false,
        returns: "BdJson",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Beads issue id",
        }],
    },
    ActionSpec {
        name: "graph.show",
        description: "Show one Beads issue dependency graph",
        destructive: false,
        returns: "BdJson",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Beads issue id",
        }],
    },
];

#[cfg(test)]
mod tests {
    use super::ACTIONS;

    #[test]
    fn beads_actions_are_read_only() {
        for spec in ACTIONS {
            assert!(!spec.destructive, "{} should remain read-only", spec.name);
        }
    }
}
