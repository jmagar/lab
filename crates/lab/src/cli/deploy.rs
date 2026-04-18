//! CLI surface for the `deploy` service.
//!
//! Thin shim over `dispatch::deploy`. Destructive actions (`run`,
//! `rollback`) require `-y` to proceed non-interactively.

use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::dispatch::deploy;
use crate::dispatch::deploy::authz::McpContext;
use crate::output::{OutputFormat, print};

#[derive(Debug, Args)]
pub struct DeployArgs {
    #[command(subcommand)]
    pub cmd: DeployCmd,
}

#[derive(Debug, Subcommand)]
pub enum DeployCmd {
    /// Show resolved deploy hosts and defaults.
    ConfigList,
    /// Dry-run: resolve targets, hash local artifact, show what would happen.
    Plan {
        /// SSH aliases to include in the plan.
        #[arg(required = true)]
        targets: Vec<String>,
    },
    /// Destructive: build, transfer, install, restart, verify.
    Run {
        /// SSH aliases to deploy to.
        #[arg(required = true)]
        targets: Vec<String>,
        /// Confirm the destructive operation (required non-interactively).
        #[arg(short = 'y', long = "yes")]
        yes: bool,
        /// Maximum number of hosts to work on concurrently.
        #[arg(long)]
        max_parallel: Option<u32>,
        /// Abort remaining hosts on the first failure.
        #[arg(long)]
        fail_fast: bool,
    },
    /// Destructive: restore the most recent backup on each target.
    Rollback {
        /// SSH aliases to roll back.
        #[arg(required = true)]
        targets: Vec<String>,
        /// Confirm the destructive operation.
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

impl DeployArgs {
    /// Targets extracted from the subcommand (empty for `config-list`).
    #[allow(dead_code)]
    #[must_use]
    pub fn cmd_targets(&self) -> Vec<String> {
        match &self.cmd {
            DeployCmd::Plan { targets }
            | DeployCmd::Run { targets, .. }
            | DeployCmd::Rollback { targets, .. } => targets.clone(),
            DeployCmd::ConfigList => vec![],
        }
    }

    /// Whether the operator passed `-y`.
    #[allow(dead_code)]
    #[must_use]
    pub fn cmd_yes(&self) -> bool {
        matches!(
            &self.cmd,
            DeployCmd::Run { yes: true, .. } | DeployCmd::Rollback { yes: true, .. }
        )
    }
}

/// Execute a deploy CLI invocation against a concrete runner.
pub async fn run<R>(args: DeployArgs, format: OutputFormat, runner: &R) -> Result<()>
where
    R: deploy::runner::DeployRunner,
{
    let (action, params) = match args.cmd {
        DeployCmd::ConfigList => ("config.list", json!({})),
        DeployCmd::Plan { targets } => ("plan", json!({ "targets": targets })),
        DeployCmd::Run {
            targets,
            yes,
            max_parallel,
            fail_fast,
        } => {
            if !yes {
                bail!("deploy run is destructive; pass -y to confirm");
            }
            (
                "run",
                json!({
                    "targets": targets,
                    "confirm": true,
                    "max_parallel": max_parallel,
                    "fail_fast": fail_fast,
                }),
            )
        }
        DeployCmd::Rollback { targets, yes } => {
            if !yes {
                bail!("deploy rollback is destructive; pass -y to confirm");
            }
            ("rollback", json!({ "targets": targets, "confirm": true }))
        }
    };

    // Scope the MCP context to CLI so authz treats this as a local operator
    // action rather than a headless MCP call.
    let value: Value = deploy::authz::MCP_CONTEXT
        .scope(
            McpContext::Cli,
            deploy::dispatch_with_runner(action, params, runner),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    print(&value, format)?;
    Ok(())
}
