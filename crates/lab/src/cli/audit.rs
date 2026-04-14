//! `lab audit` — onboarding audit.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::audit::{AuditReport, audit_services};
use crate::output::{OutputFormat, print, render_audit_report};

/// `lab audit` arguments.
#[derive(Debug, Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub command: AuditCommand,
}

/// `lab audit` subcommands.
#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Audit onboarding for one or more services.
    Onboarding(OnboardingArgs),
}

/// `lab audit onboarding` arguments.
#[derive(Debug, Args)]
pub struct OnboardingArgs {
    /// Services to audit.
    #[arg(required = true)]
    pub services: Vec<String>,
}

#[allow(clippy::print_stdout)]
pub fn run(args: AuditArgs, format: OutputFormat) -> Result<ExitCode> {
    match args.command {
        AuditCommand::Onboarding(args) => {
            let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .ok_or_else(|| {
                    anyhow::anyhow!("cannot determine workspace root from CARGO_MANIFEST_DIR")
                })?
                .to_path_buf();
            let report = audit_services(&args.services, &repo_root);
            match format {
                OutputFormat::Json => print(&report, format)?,
                OutputFormat::Human => println!("{}", render_audit_report(&report)),
            }
            Ok(exit_code(&report))
        }
    }
}

fn exit_code(report: &AuditReport) -> ExitCode {
    if report.has_failures() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{CheckResult, ServiceReport};

    #[test]
    fn audit_exit_code_fails_on_failed_checks() {
        let report = AuditReport {
            services: vec![ServiceReport {
                service: "sampleaudit".into(),
                checks: vec![("docs.coverage".into(), CheckResult::Fail("missing".into()))],
            }],
        };
        assert_eq!(exit_code(&report), ExitCode::FAILURE);
    }
}
