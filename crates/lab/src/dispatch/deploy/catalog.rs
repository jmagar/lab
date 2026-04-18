//! Action catalog for the `deploy` service.

use lab_apis::core::action::ActionSpec;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "List deploy actions",
        destructive: false,
        params: &[],
        returns: "Catalog",
    },
    ActionSpec {
        name: "schema",
        description: "Per-action JSON schema",
        destructive: false,
        params: &[],
        returns: "Schema",
    },
    ActionSpec {
        name: "config.list",
        description: "Show resolved deploy hosts and defaults",
        destructive: false,
        params: &[],
        returns: "ConfigListing",
    },
    ActionSpec {
        name: "plan",
        description: "Dry-run: resolve targets, hash local artifact, show what would happen",
        destructive: false,
        params: &[],
        returns: "DeployPlan",
    },
    ActionSpec {
        name: "run",
        description: "Build, transfer, install, restart, verify on targets (destructive)",
        destructive: true,
        params: &[],
        returns: "DeployRunSummary",
    },
    ActionSpec {
        name: "rollback",
        description: "Restore the most recent timestamped backup on the specified targets (destructive)",
        destructive: true,
        params: &[],
        returns: "DeployRunSummary",
    },
];
