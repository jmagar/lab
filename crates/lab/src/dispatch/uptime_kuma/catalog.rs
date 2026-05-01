use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `uptime-kuma` service.
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
        description: "Show the Uptime Kuma integration contract and live action status",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "server.health",
        description: "Probe the Uptime Kuma web UI root",
        destructive: false,
        returns: "null",
        params: &[],
    },
    ActionSpec {
        name: "monitor.list",
        description: "List monitors through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "monitor.get",
        description: "Fetch one monitor through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Monitor id",
        }],
    },
    ActionSpec {
        name: "monitor.create",
        description: "Create a monitor through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "config",
            ty: "object",
            required: true,
            description: "Uptime Kuma monitor config object",
        }],
    },
    ActionSpec {
        name: "monitor.update",
        description: "Update a monitor through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Monitor id",
            },
            ParamSpec {
                name: "config",
                ty: "object",
                required: true,
                description: "Uptime Kuma monitor config object",
            },
        ],
    },
    ActionSpec {
        name: "monitor.delete",
        description: "Delete a monitor through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Monitor id",
        }],
    },
    ActionSpec {
        name: "monitor.pause",
        description: "Pause monitor checks through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Monitor id",
        }],
    },
    ActionSpec {
        name: "monitor.resume",
        description: "Resume monitor checks through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Monitor id",
        }],
    },
    ActionSpec {
        name: "heartbeat.list",
        description: "List heartbeat rows through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[
            ParamSpec {
                name: "monitor_id",
                ty: "integer",
                required: true,
                description: "Monitor id",
            },
            ParamSpec {
                name: "hours",
                ty: "integer",
                required: false,
                description: "History window in hours, default 24, max 168",
            },
        ],
    },
    ActionSpec {
        name: "status.summary",
        description: "Summarize monitor status through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "notification.list",
        description: "List notification channels through the Socket.IO actor",
        destructive: false,
        returns: "UptimeKumaResponse",
        params: &[],
    },
    ActionSpec {
        name: "notification.test",
        description: "Send a real notification through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "notification_id",
            ty: "integer",
            required: true,
            description: "Notification channel id to test",
        }],
    },
    ActionSpec {
        name: "notification.add",
        description: "Create a notification channel through the Socket.IO actor",
        destructive: true,
        returns: "UptimeKumaResponse",
        params: &[ParamSpec {
            name: "config",
            ty: "object",
            required: true,
            description: "Notification config object; template markers and local webhook targets are rejected",
        }],
    },
];

#[cfg(test)]
mod tests {
    use super::ACTIONS;

    #[test]
    fn uptime_kuma_mutations_are_destructive() {
        for action in [
            "monitor.create",
            "monitor.update",
            "monitor.delete",
            "monitor.pause",
            "monitor.resume",
            "notification.test",
            "notification.add",
        ] {
            let spec = ACTIONS.iter().find(|spec| spec.name == action).unwrap();
            assert!(spec.destructive, "{action} should be destructive");
        }
    }

    #[test]
    fn uptime_kuma_has_full_action_surface() {
        let service_actions = ACTIONS
            .iter()
            .filter(|spec| spec.name != "help" && spec.name != "schema")
            .count();
        assert_eq!(service_actions, 14);
    }
}
