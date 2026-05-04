//! `labby docs` — generated documentation artifacts.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::docs;

#[derive(Debug, Args)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: DocsCommand,
}

#[derive(Debug, Subcommand)]
pub enum DocsCommand {
    /// Regenerate every tracked generated-docs artifact.
    Generate,
    /// Verify generated-docs artifacts are fresh.
    Check,
}

#[allow(clippy::print_stdout)]
pub fn run(args: DocsArgs) -> Result<ExitCode> {
    match args.command {
        DocsCommand::Generate => {
            let outcome = docs::generate()?;
            println!("generated {} docs artifacts", outcome.checked);
            Ok(ExitCode::SUCCESS)
        }
        DocsCommand::Check => {
            let outcome = docs::check()?;
            if outcome.stale.is_empty() {
                println!("checked {} docs artifacts: fresh", outcome.checked);
                Ok(ExitCode::SUCCESS)
            } else {
                for path in &outcome.stale {
                    println!("stale: {path}");
                }
                Ok(ExitCode::FAILURE)
            }
        }
    }
}
