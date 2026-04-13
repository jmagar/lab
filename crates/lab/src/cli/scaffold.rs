//! `lab scaffold` — generate a new service onboarding skeleton.

use std::io::IsTerminal;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::output::{OutputFormat, print, render_scaffold_result};
use crate::scaffold::{ScaffoldConfig, ScaffoldKind};

/// `lab scaffold` arguments.
#[derive(Debug, Args)]
pub struct ScaffoldArgs {
    #[command(subcommand)]
    pub command: ScaffoldCommand,
}

/// `lab scaffold` subcommands.
#[derive(Debug, Subcommand)]
pub enum ScaffoldCommand {
    /// Scaffold one service.
    Service(ServiceArgs),
}

/// `lab scaffold service` arguments.
#[derive(Debug, Args)]
pub struct ServiceArgs {
    /// Service name.
    pub service: String,

    /// Service kind.
    #[arg(long, value_enum, default_value_t = ScaffoldKind::Http)]
    pub kind: ScaffoldKind,

    /// Only show the planned file operations.
    #[arg(long)]
    pub dry_run: bool,

    /// Confirm that scaffold writes are allowed.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Run `lab scaffold`.
pub fn run(args: ScaffoldArgs, format: OutputFormat) -> Result<ExitCode> {
    match args.command {
        ScaffoldCommand::Service(args) => run_service(args, format),
    }
}

#[allow(clippy::print_stdout, clippy::print_stderr)]
fn run_service(args: ServiceArgs, format: OutputFormat) -> Result<ExitCode> {
    if !args.dry_run && !args.yes {
        if std::io::stdin().is_terminal() {
            eprint!(
                "scaffold will write files for service `{}`. Continue? [y/N] ",
                args.service
            );
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            if !matches!(line.trim(), "y" | "Y") {
                anyhow::bail!("scaffold cancelled");
            }
        } else {
            anyhow::bail!("pass -y / --yes to confirm scaffold writes");
        }
    }

    let config = ScaffoldConfig {
        service: args.service,
        kind: args.kind,
        repo_root: std::env::current_dir()?,
    };
    let result = crate::scaffold::scaffold_service(&config, args.dry_run)?;

    match format {
        OutputFormat::Json => print(&result, format)?,
        OutputFormat::Human => {
            println!("{}", render_scaffold_result(&result));
        }
    }

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    fn config(root: PathBuf) -> ScaffoldConfig {
        ScaffoldConfig {
            service: "sampleaudit".to_string(),
            kind: ScaffoldKind::Http,
            repo_root: root,
        }
    }

    fn seed_repo(root: &std::path::Path) {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("repo root");
        for rel in [
            "crates/lab-apis/Cargo.toml",
            "crates/lab/Cargo.toml",
            "crates/lab-apis/src/lib.rs",
            "crates/lab/src/dispatch.rs",
            "crates/lab/src/cli.rs",
            "crates/lab/src/mcp/services.rs",
            "crates/lab/src/mcp/registry.rs",
            "crates/lab/src/api/services.rs",
            "crates/lab/src/api/router.rs",
            "crates/lab/src/api/state.rs",
            "crates/lab/src/tui/metadata.rs",
        ] {
            let src = repo_root.join(rel);
            let dst = root.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).expect("fixture parent");
            }
            fs::copy(src, dst).expect("fixture copy");
        }
    }

    #[test]
    fn scaffold_dry_run_plans_files_without_writing() {
        let dir = tempdir().expect("tempdir");
        seed_repo(dir.path());
        let result = crate::scaffold::scaffold_service(&config(dir.path().to_path_buf()), true)
            .expect("scaffold");

        assert!(result.dry_run);
        assert_eq!(result.service, "sampleaudit");
        assert!(
            result
                .planned
                .iter()
                .any(|op| op.path == PathBuf::from("crates/lab-apis/src/sampleaudit.rs"))
        );
        assert!(
            !dir.path()
                .join("crates/lab-apis/src/sampleaudit.rs")
                .exists(),
            "dry-run should not write files"
        );
    }

    #[test]
    fn scaffold_repeated_scaffold_is_idempotent() {
        let dir = tempdir().expect("tempdir");
        seed_repo(dir.path());
        let root = dir.path().to_path_buf();

        let first =
            crate::scaffold::scaffold_service(&config(root.clone()), false).expect("first scaffold");
        assert!(!first.created.is_empty());

        let second =
            crate::scaffold::scaffold_service(&config(root), false).expect("second scaffold");
        assert!(second.created.is_empty());
        assert!(second.modified.is_empty());
    }
}
