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
        name: "system.checks",
        description: "Run local system probes: env vars, Docker, disk, ports, config files",
        destructive: false,
        returns: "DoctorReport",
        params: &[],
    },
    ActionSpec {
        name: "service.probe",
        description: "Probe a single named service via its health endpoint",
        destructive: false,
        returns: "Finding",
        params: &[
            ParamSpec {
                name: "service",
                ty: "string",
                required: true,
                description: "Service name to probe (e.g. \"radarr\", \"sonarr\")",
            },
            ParamSpec {
                name: "instance",
                ty: "string",
                required: false,
                description: "Named instance label for multi-instance services",
            },
        ],
    },
    ActionSpec {
        name: "audit.full",
        description: "Probe all configured services plus system checks; streams results",
        destructive: false,
        returns: "stream<Finding>",
        params: &[],
    },
];
