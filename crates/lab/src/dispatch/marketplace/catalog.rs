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
        name: "plugin.workspace",
        description: "Load or create an app-managed editable workspace mirror for a plugin",
        destructive: false,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "PluginWorkspace",
    },
    ActionSpec {
        name: "plugin.save",
        description: "Save a file into the plugin workspace mirror",
        destructive: false,
        params: &[
            ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Plugin id in `name@marketplace` form",
            },
            ParamSpec {
                name: "path",
                ty: "string",
                required: true,
                description: "Relative file path inside the plugin workspace",
            },
            ParamSpec {
                name: "content",
                ty: "string",
                required: true,
                description: "Updated file contents",
            },
        ],
        returns: "SaveResult",
    },
    ActionSpec {
        name: "plugin.deploy",
        description: "Deploy the saved plugin workspace to the local Claude Code install target",
        destructive: true,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "DeployResult",
    },
    ActionSpec {
        name: "plugin.deploy.preview",
        description: "Preview changed, skipped, and removed files before deploying the workspace",
        destructive: false,
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Plugin id in `name@marketplace` form",
        }],
        returns: "DeployPreviewResult",
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
            ParamSpec {
                name: "autoUpdate",
                ty: "boolean",
                required: false,
                description: "Persist whether this marketplace should auto-update",
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
    // ── ACP agent actions (lab-zxx5.3) ───────────────────────────────────
    // Mirrors `acp_catalog::ACP_ACTIONS`; this catalog drives `help`/`schema`
    // for the marketplace MCP/CLI/API surface.
    ActionSpec {
        name: "agent.list",
        description: "List ACP-compatible agents from the registry CDN",
        destructive: false,
        returns: "Agent[]",
        params: &[],
    },
    ActionSpec {
        name: "agent.get",
        description: "Get details for a single ACP agent by id",
        destructive: false,
        returns: "Agent",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Agent id (e.g. `anthropic/claude-code`)",
        }],
    },
    ActionSpec {
        name: "agent.install",
        description: "Install an ACP agent on one or more devices. Writes a provider entry to `~/.lab/acp-providers.json`. Binary distributions are not yet supported in this build; npx/uvx are config-only.",
        destructive: true,
        returns: "InstallResults",
        params: &[
            ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Agent id from the registry",
            },
            ParamSpec {
                name: "node_ids",
                ty: "array",
                required: true,
                description: "Node ids to install on (`\"local\"` for the controller host)",
            },
            ParamSpec {
                name: "platform",
                ty: "string",
                required: false,
                description: "Override platform triple for binary lookup (e.g. `linux-x86_64`)",
            },
            ParamSpec {
                name: "confirm",
                ty: "boolean",
                required: true,
                description: "Must be true to confirm the destructive install operation",
            },
        ],
    },
    // ── Plugin cherry-pick (lab-zxx5.6) ──────────────────────────────────────
    ActionSpec {
        name: "plugin.cherry_pick",
        description: "Install selected components from a plugin to one or more devices",
        destructive: true,
        params: &[
            ParamSpec {
                name: "plugin_id",
                ty: "string",
                required: true,
                description: "Plugin id in `name@marketplace` form",
            },
            ParamSpec {
                name: "components",
                ty: "array",
                required: true,
                description: "Component paths to install (e.g. `agents/my-agent.md`)",
            },
            ParamSpec {
                name: "node_ids",
                ty: "array",
                required: true,
                description: "Target node ids (`\"local\"` for the controller host)",
            },
            ParamSpec {
                name: "scope",
                ty: "string",
                required: true,
                description: "`global` (to `~/.claude/`) or `project` (to `project_path/.claude/`)",
            },
            ParamSpec {
                name: "project_path",
                ty: "string",
                required: false,
                description: "Absolute project path — required when `scope` is `project`",
            },
            ParamSpec {
                name: "confirm",
                ty: "boolean",
                required: true,
                description: "Must be `true` to confirm this destructive operation",
            },
        ],
        returns: "CherryPickResults",
    },
    ActionSpec {
        name: "agent.uninstall",
        description: "Remove an installed ACP agent entry from `~/.lab/acp-providers.json`",
        destructive: true,
        returns: "UninstallResult",
        params: &[
            ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Agent id to uninstall",
            },
            ParamSpec {
                name: "confirm",
                ty: "boolean",
                required: true,
                description: "Must be true to confirm the destructive uninstall operation",
            },
        ],
    },
];
