//! ActionSpec catalog for `agent.*` actions in the marketplace dispatch.
//!
//! These actions wrap the `lab-apis::acp_registry` SDK to discover, install,
//! and remove ACP-compatible AI coding agents.

use lab_apis::core::action::{ActionSpec, ParamSpec};

pub const ACP_ACTIONS: &[ActionSpec] = &[
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
        description: "Install an ACP agent on one or more devices. Writes a provider entry to `~/.lab/acp-providers.json`. Binary distributions are not yet supported in this build; npx/uvx distributions are config-only.",
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
                name: "device_ids",
                ty: "array",
                required: true,
                description: "Device ids to install on. Use `\"local\"` for the controller host. Remote installs are not yet implemented and return per-device errors.",
            },
            ParamSpec {
                name: "platform",
                ty: "string",
                required: false,
                description: "Override platform triple for binary lookup (e.g. `linux-x86_64`); auto-detected from the host when omitted.",
            },
            ParamSpec {
                name: "confirm",
                ty: "boolean",
                required: true,
                description: "Must be true to confirm the destructive install operation",
            },
        ],
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
