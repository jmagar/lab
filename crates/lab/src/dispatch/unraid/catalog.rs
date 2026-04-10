//! Action catalog for the `unraid` service.
//!
//! Single authoritative source for MCP, CLI, and API surfaces.

use lab_apis::core::action::{ActionSpec, ParamSpec};

/// All actions exposed by the `unraid` tool.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "system.info",
        description: "Return detailed system information (OS, CPU, versions)",
        destructive: false,
        returns: "SystemInfo",
        params: &[],
    },
    ActionSpec {
        name: "system.metrics",
        description: "Return current CPU and memory utilisation metrics",
        destructive: false,
        returns: "SystemMetrics",
        params: &[],
    },
    ActionSpec {
        name: "system.array",
        description: "Return array state and disk list (data, parity, cache)",
        destructive: false,
        returns: "ArrayStatus",
        params: &[],
    },
    ActionSpec {
        name: "system.online",
        description: "Return whether the Unraid server reports itself as online",
        destructive: false,
        returns: "object: { online: bool }",
        params: &[],
    },
    ActionSpec {
        name: "docker.list",
        description: "List all Docker containers",
        destructive: false,
        returns: "Vec<DockerContainer>",
        params: &[],
    },
    ActionSpec {
        name: "docker.start",
        description: "Start a Docker container",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID (e.g. docker_container:abc123)",
        }],
    },
    ActionSpec {
        name: "docker.stop",
        description: "Stop a Docker container",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID",
        }],
    },
    ActionSpec {
        name: "docker.restart",
        description: "Restart a Docker container (stop then start)",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Container prefixed ID",
        }],
    },
    ActionSpec {
        name: "disk.list",
        description: "List physical disks attached to the server",
        destructive: false,
        returns: "Vec<DiskInfo>",
        params: &[],
    },
];
