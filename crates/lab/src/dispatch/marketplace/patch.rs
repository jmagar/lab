//! Marketplace artifact diff/patch stubs.
//!
//! Full diff and patch behavior belongs to `lab-iut1.4`. This module wires the
//! action surface with stable signatures and structured placeholder errors.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::marketplace::params::{ArtifactDiffParams, PatchParams};

pub(super) async fn artifact_diff(params: ArtifactDiffParams) -> Result<Value, ToolError> {
    Err(not_implemented_error(
        "artifact.diff",
        format!(
            "artifact diff is not implemented yet for `{}`",
            params.plugin_id
        ),
    ))
}

pub(super) async fn artifact_patch(params: PatchParams) -> Result<Value, ToolError> {
    Err(not_implemented_error(
        "artifact.patch",
        format!(
            "artifact patch is not implemented yet for `{}` at `{}`",
            params.plugin_id, params.artifact_path
        ),
    ))
}

fn not_implemented_error(action: &'static str, detail: String) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "not_implemented".to_string(),
        message: format!("{action}: {detail}"),
    }
}
