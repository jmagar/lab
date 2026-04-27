//! Action catalog for the `beads` service.
//!
//! V1 catalog: read-only. Deferred actions (until v2):
//!   issue.create, issue.update, issue.close, issue.reopen,
//!   comment.list, comment.add, label.list

use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog — read-only v1.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "issue.list",
        description: "List issues with optional filters",
        destructive: false,
        returns: "IssueSummary[]",
        params: &[
            ParamSpec {
                name: "status",
                ty: "string",
                required: false,
                description: "Filter by status (open/closed/in_progress)",
            },
            ParamSpec {
                name: "issue_type",
                ty: "string",
                required: false,
                description: "Filter by type (task/epic/bug/feature/chore)",
            },
            ParamSpec {
                name: "owner",
                ty: "string",
                required: false,
                description: "Filter by owner",
            },
            ParamSpec {
                name: "label",
                ty: "string",
                required: false,
                description: "Filter by label",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Max results (default 50, max 200)",
            },
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Pagination offset (default 0)",
            },
        ],
    },
    ActionSpec {
        name: "issue.get",
        description: "Get full issue detail (with labels) by id",
        destructive: false,
        returns: "Issue",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Issue id (e.g. lab-5t4b)",
        }],
    },
];
