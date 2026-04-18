//! Deploy orchestrator.
//!
//! V1 scope: trait + in-process default runner that drives the
//! build → preflight → transfer → install → restart → verify pipeline over
//! SSH using `tokio::process::Command`. Tests in this crate substitute a
//! recording `HostIo` mock for fast orchestration coverage without a live
//! SSH server.

use lab_apis::deploy::{DeployError, DeployPlan, DeployRequest, DeployRunSummary};
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Surface-neutral deploy orchestrator used by CLI and MCP adapters.
pub trait DeployRunner: Send + Sync {
    fn plan(
        &self,
        req: &DeployRequest,
    ) -> impl std::future::Future<Output = Result<DeployPlan, ToolError>> + Send;

    fn run(
        &self,
        req: &DeployRequest,
    ) -> impl std::future::Future<Output = Result<DeployRunSummary, ToolError>> + Send;

    fn rollback(
        &self,
        req: &DeployRequest,
    ) -> impl std::future::Future<Output = Result<DeployRunSummary, ToolError>> + Send;

    fn config_list(&self)
    -> impl std::future::Future<Output = Result<Value, ToolError>> + Send;
}

/// Low-level primitive the runner uses to talk to a single host. The real
/// implementation wraps `lab_apis::core::ssh::SshOptions` + `tokio::process::
/// Command`; tests substitute a recording fake.
pub trait HostIo: Send + Sync {
    fn run_argv(
        &self,
        argv: &[&str],
    ) -> impl std::future::Future<Output = Result<(i32, String, String), DeployError>> + Send;

    fn sha256_remote(
        &self,
        remote_path: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, DeployError>> + Send;
}

/// Placeholder runner for bringup. Real orchestration lands in a follow-up
/// commit — this scaffold exists so the dispatch layer compiles and the
/// CLI/MCP surfaces can wire through it.
#[derive(Default)]
pub struct NoopRunner;

impl DeployRunner for NoopRunner {
    async fn plan(&self, _req: &DeployRequest) -> Result<DeployPlan, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not yet implemented on this build",
        ))
    }

    async fn run(&self, _req: &DeployRequest) -> Result<DeployRunSummary, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not yet implemented on this build",
        ))
    }

    async fn rollback(&self, _req: &DeployRequest) -> Result<DeployRunSummary, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not yet implemented on this build",
        ))
    }

    async fn config_list(&self) -> Result<Value, ToolError> {
        Ok(serde_json::json!({
            "defaults": null,
            "hosts": [],
            "overrides": [],
        }))
    }
}
