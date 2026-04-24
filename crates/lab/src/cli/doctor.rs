//! `lab doctor` — thin shim over the doctor dispatch service.
//!
//! Exit codes: 0 = ok, 1 = warnings, 2 = failures.

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Result;

use crate::dispatch::clients::ServiceClients;
use crate::dispatch::doctor::{Finding, Report, Severity};
use crate::output::OutputFormat;

pub use crate::dispatch::doctor::service_env_checks;


/// Run the doctor subcommand, streaming results as they arrive.
pub async fn run(format: OutputFormat) -> Result<ExitCode> {
    use tokio::sync::mpsc;
    let clients = Arc::new(ServiceClients::from_env());
    let (tx, mut rx) = mpsc::channel(64);

    tokio::spawn(async move {
        crate::dispatch::doctor::service::stream_audit_full(clients, tx).await;
    });

    let mut findings: Vec<Finding> = Vec::new();

    if format.is_json() {
        while let Some(f) = rx.recv().await {
            findings.push(f);
        }
        let report = Report { findings };
        let json = serde_json::to_string_pretty(&report)
            .map_err(|e| anyhow::anyhow!("serialization error: {e}"))?;
        println!("{json}");
        Ok(exit_code(&report))
    } else {
        while let Some(f) = rx.recv().await {
            let icon = match f.severity {
                Severity::Ok => "✓",
                Severity::Warn => "⚠",
                Severity::Fail => "✗",
            };
            println!("{icon} [{service}] {check}: {msg}",
                service = f.service,
                check = f.check,
                msg = f.message);
            findings.push(f);
        }
        let report = Report { findings };
        Ok(exit_code(&report))
    }
}

fn exit_code(report: &Report) -> ExitCode {
    match report.worst() {
        Severity::Ok => ExitCode::SUCCESS,
        Severity::Warn => ExitCode::from(1),
        Severity::Fail => ExitCode::from(2),
    }
}

#[cfg(test)]
mod tests {
    use super::service_env_checks;

    #[test]
    fn extract_is_always_in_checks() {
        let checks = service_env_checks();
        assert!(checks.iter().any(|(name, _)| *name == "extract"));
    }

    #[test]
    #[cfg(feature = "radarr")]
    fn radarr_in_checks_when_feature_enabled() {
        let checks = service_env_checks();
        assert!(checks.iter().any(|(name, _)| *name == "radarr"));
    }
}
