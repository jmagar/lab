//! Top-level CLI — clap derive definitions and dispatch router.
//!
//! Every subcommand is a thin shim that parses args, calls into a
//! `lab-apis` client (or a lab-local subsystem), and formats output.
//! See `crates/lab/src/cli/CLAUDE.md` for the rulebook.

pub mod completions;
pub mod doctor;
pub mod health;
pub mod help;
pub mod install;
pub mod plugins;
pub mod serve;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

/// `lab` — pluggable homelab CLI + MCP server SDK.
#[derive(Debug, Parser)]
#[command(name = "lab", version, about, long_about = None, disable_help_subcommand = true)]
pub struct Cli {
    /// Emit JSON instead of human-readable tables.
    #[arg(long, global = true)]
    pub json: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Resolved output format based on the `--json` flag.
    #[must_use]
    pub const fn format(&self) -> OutputFormat {
        OutputFormat::from_json_flag(self.json)
    }
}

/// Every top-level subcommand. Service subcommands are added in later
/// plans as each service comes online.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the MCP server (stdio or HTTP transport).
    Serve(serve::ServeArgs),
    /// Audit configured services and report problems.
    Doctor,
    /// Quick reachability check for configured services.
    Health,
    /// Open the plugin manager TUI.
    Plugins,
    /// Install one or more services into `.mcp.json`.
    Install(install::InstallArgs),
    /// Uninstall services from `.mcp.json`.
    Uninstall(install::UninstallArgs),
    /// First-time setup wizard.
    Init,
    /// Print the service + action catalog.
    Help,
    /// Generate shell completions.
    Completions(completions::CompletionsArgs),
}

/// Dispatch a parsed [`Cli`] to the correct handler.
pub async fn dispatch(cli: Cli) -> Result<ExitCode> {
    let format = cli.format();
    match cli.command {
        Command::Serve(args) => serve::run(args).await,
        Command::Doctor => doctor::run(format).await,
        Command::Health => health::run(format).await,
        Command::Plugins => plugins::run(),
        Command::Install(args) => install::run_install(args),
        Command::Uninstall(args) => install::run_uninstall(args),
        Command::Init => install::run_init(),
        Command::Help => help::run(format),
        Command::Completions(args) => completions::run(args),
    }
}
