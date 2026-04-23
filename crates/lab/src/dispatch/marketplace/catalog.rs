use lab_apis::core::action::{ActionSpec, ParamSpec};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        params: &[],
        returns: "Catalog",
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
        returns: "Schema",
    },
    ActionSpec {
        name: "sources.list",
        description: "List configured marketplaces",
        destructive: false,
        params: &[],
        returns: "Marketplace[]",
    },
    ActionSpec {
        name: "plugins.list",
        description: "List all plugins across marketplaces with installed state",
        destructive: false,
        params: &[ParamSpec {
            name: "marketplace",
            ty: "string",
            required: false,
            description: "Filter to a single marketplace id",
        }],
        returns: "Plugin[]",
    },
    ActionSpec {
        name: "plugin.get",
        description: "Return a single plugin by id (`name@marketplace`)",
        destructive: false,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "Plugin",
    },
    ActionSpec {
        name: "plugin.artifacts",
        description: "List artifact files shipped with an installed plugin",
        destructive: false,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "Artifact[]",
    },
    ActionSpec {
        name: "sources.add",
        description: "Register a new marketplace via `claude plugin marketplace add`",
        destructive: true,
        params: &[
            ParamSpec {
                name: "repo",
                ty: "string",
                required: false,
                description: "GitHub `owner/repo` slug (mutually exclusive with `url`)",
            },
            ParamSpec {
                name: "url",
                ty: "string",
                required: false,
                description: "Git URL (mutually exclusive with `repo`)",
            },
        ],
        returns: "AddResult",
    },
    ActionSpec {
        name: "plugin.install",
        description: "Install a plugin via `claude plugin install`",
        destructive: true,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "InstallResult",
    },
    ActionSpec {
        name: "plugin.uninstall",
        description: "Uninstall a plugin via `claude plugin uninstall`",
        destructive: true,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "UninstallResult",
    },
];
