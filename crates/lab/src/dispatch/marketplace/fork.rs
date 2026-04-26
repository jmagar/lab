//! Marketplace artifact fork lifecycle stubs.
//!
//! Full fork lifecycle behavior belongs to `lab-iut1.3`. This module wires the
//! action surface with stable signatures and structured placeholder errors.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::marketplace::params::{
    ArtifactListParams, ArtifactResetParams, ForkParams, UnforkParams,
};

pub(super) async fn artifact_fork(params: ForkParams) -> Result<Value, ToolError> {
    Err(not_implemented_error(
        "artifact.fork",
        format!(
            "fork lifecycle is not implemented yet for `{}`",
            params.plugin_id
        ),
    ))
}

pub(super) async fn artifact_list(params: ArtifactListParams) -> Result<Value, ToolError> {
    Err(not_implemented_error(
        "artifact.list",
        params
            .plugin_id
            .map(|plugin_id| format!("fork listing is not implemented yet for `{plugin_id}`"))
            .unwrap_or_else(|| "fork listing is not implemented yet".to_string()),
    ))
}

pub(super) async fn artifact_unfork(params: UnforkParams) -> Result<Value, ToolError> {
    tracing::info!(
        surface = "dispatch",
        service = "marketplace",
        action = "artifact.unfork",
        plugin_id = %params.plugin_id,
        "destructive action intent: removing fork metadata"
    );
    Err(not_implemented_error(
        "artifact.unfork",
        format!(
            "fork lifecycle removal is not implemented yet for `{}`",
            params.plugin_id
        ),
    ))
}

pub(super) async fn artifact_reset(params: ArtifactResetParams) -> Result<Value, ToolError> {
    tracing::info!(
        surface = "dispatch",
        service = "marketplace",
        action = "artifact.reset",
        plugin_id = %params.plugin_id,
        "destructive action intent: resetting forked artifacts"
    );
    Err(not_implemented_error(
        "artifact.reset",
        format!(
            "fork reset is not implemented yet for `{}`",
            params.plugin_id
        ),
    ))
}

fn not_implemented_error(action: &'static str, detail: String) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "not_implemented".to_string(),
        message: format!("{action}: {detail}"),
    }
}
