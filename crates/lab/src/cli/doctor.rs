//! `lab doctor` — comprehensive health audit.
//!
//! Exit codes: 0 = ok, 1 = warnings, 2 = failures. Real checks wired in
//! a later plan; this stub always returns 0 with an empty report.

use std::process::ExitCode;

use anyhow::Result;
use serde::Serialize;

use crate::output::{OutputFormat, print};

/// Severity of a single doctor finding.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// All good.
    Ok,
    /// Non-fatal issue.
    Warn,
    /// Hard failure.
    Fail,
}

/// One entry in the doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Service the finding applies to.
    pub service: String,
    /// Check name (e.g., `env_present`, `reachable`).
    pub check: String,
    /// Severity bucket.
    pub severity: Severity,
    /// Human-readable detail.
    pub message: String,
}

/// Full doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// All findings, in scan order.
    pub findings: Vec<Finding>,
}

/// Run the doctor subcommand.
pub async fn run(format: OutputFormat) -> Result<ExitCode> {
    let report = Report { findings: Vec::new() };
    print(&report, format)?;

    let worst = report.findings.iter().map(|f| f.severity).fold(
        Severity::Ok,
        |acc, s| match (acc, s) {
            (Severity::Fail, _) | (_, Severity::Fail) => Severity::Fail,
            (Severity::Warn, _) | (_, Severity::Warn) => Severity::Warn,
            _ => Severity::Ok,
        },
    );

    Ok(match worst {
        Severity::Ok => ExitCode::SUCCESS,
        Severity::Warn => ExitCode::from(1),
        Severity::Fail => ExitCode::from(2),
    })
}
