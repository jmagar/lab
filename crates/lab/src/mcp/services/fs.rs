//! MCP adapter for the `fs` workspace filesystem browser service.
//!
//! The catalog and dispatch logic live in `crates/lab/src/dispatch/fs/`.
//!
//! # Why this file filters `fs.preview` out
//!
//! `crate::dispatch::fs::ACTIONS` contains both `fs.list` and `fs.preview`
//! — the latter is intentionally **not** exposed over MCP. An LLM agent
//! constructs the request body, so no param-based confirmation gate is
//! safe against prompt injection: an attacker who gets a prompt-injection
//! payload into the session can ask the agent to `fs.preview(".env")` and
//! exfiltrate the file in one round-trip. The deny-list is defense-in-depth
//! but not a sound boundary; the only safe policy is "HTTP-only".
//!
//! This mirrors the `api/CLAUDE.md` decision to remove the `X-Lab-Confirm`
//! header confirmation path.

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// MCP-exposed slice of the fs action catalog. Filters out `fs.preview`.
pub static ACTIONS: &[ActionSpec] = MCP_ACTIONS;

/// Compile-time filtered view of [`crate::dispatch::fs::ACTIONS`].
///
/// We cannot slice a `&'static [ActionSpec]` at runtime into another
/// `&'static` slice, so we redeclare the MCP-visible entries here. Any
/// change to the canonical catalog must be mirrored — a unit test in this
/// module enforces the invariant that every entry here exists in
/// `dispatch::fs::ACTIONS`.
static MCP_ACTIONS: &[ActionSpec] = &[
    // Keep in sync with `dispatch::fs::catalog::ACTIONS[0]`.
    ActionSpec {
        name: "fs.list",
        description: "List immediate entries of a directory inside the configured workspace root",
        destructive: false,
        params: &[lab_apis::core::action::ParamSpec {
            name: "path",
            ty: "string",
            required: false,
            description: "Workspace-relative path to list; empty or omitted means the workspace root",
        }],
        returns: "{entries: [{name, path, kind, size, modified, accessible}], truncated: bool}",
    },
];

/// MCP dispatch entry point. `fs.preview` is rejected here as an additional
/// defense on top of the filtered `ACTIONS` slice — if tool-registration
/// ever accidentally surfaces the action, the dispatcher still refuses.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    if action == "fs.preview" {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: "fs.preview is HTTP-only; call GET /v1/fs/preview instead".to_string(),
        });
    }
    crate::dispatch::fs::dispatch(action, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_actions_exclude_fs_preview() {
        let names: Vec<&str> = MCP_ACTIONS.iter().map(|a| a.name).collect();
        assert!(!names.contains(&"fs.preview"), "{names:?}");
        assert!(names.contains(&"fs.list"));
    }

    #[test]
    fn mcp_actions_subset_of_canonical() {
        let canonical: Vec<&str> = crate::dispatch::fs::ACTIONS
            .iter()
            .map(|a| a.name)
            .collect();
        for a in MCP_ACTIONS {
            assert!(
                canonical.contains(&a.name),
                "MCP action `{}` missing from canonical catalog",
                a.name
            );
        }
    }

    #[tokio::test]
    async fn dispatch_rejects_fs_preview() {
        let err = dispatch("fs.preview", serde_json::json!({"path": "foo"}))
            .await
            .expect_err("err");
        match err {
            ToolError::Sdk { sdk_kind, .. } => assert_eq!(sdk_kind, "not_found"),
            other => panic!("expected Sdk not_found; got {other:?}"),
        }
    }
}
