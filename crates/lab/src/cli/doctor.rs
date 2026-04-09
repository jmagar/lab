//! `lab doctor` — comprehensive health audit.
//!
//! Exit codes: 0 = ok, 1 = warnings, 2 = failures.

use std::process::ExitCode;

use anyhow::Result;
use lab_apis::core::plugin::EnvVar;
use serde::Serialize;

use crate::output::{OutputFormat, print};

/// Severity of a single doctor finding.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Ok,
    Warn,
    Fail,
}

/// One entry in the doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub service: String,
    pub check: String,
    pub severity: Severity,
    pub message: String,
}

/// Full doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
}

/// Returns `(service_name, required_env_vars)` for every enabled service.
#[allow(clippy::too_many_lines)]
pub fn service_env_checks() -> Vec<(&'static str, &'static [EnvVar])> {
    let mut list = vec![(
        lab_apis::extract::META.name,
        lab_apis::extract::META.required_env,
    )];

    #[cfg(feature = "radarr")]
    list.push((
        lab_apis::radarr::META.name,
        lab_apis::radarr::META.required_env,
    ));
    #[cfg(feature = "sonarr")]
    list.push((
        lab_apis::sonarr::META.name,
        lab_apis::sonarr::META.required_env,
    ));
    #[cfg(feature = "prowlarr")]
    list.push((
        lab_apis::prowlarr::META.name,
        lab_apis::prowlarr::META.required_env,
    ));
    #[cfg(feature = "plex")]
    list.push((lab_apis::plex::META.name, lab_apis::plex::META.required_env));
    #[cfg(feature = "tautulli")]
    list.push((
        lab_apis::tautulli::META.name,
        lab_apis::tautulli::META.required_env,
    ));
    #[cfg(feature = "sabnzbd")]
    list.push((
        lab_apis::sabnzbd::META.name,
        lab_apis::sabnzbd::META.required_env,
    ));
    #[cfg(feature = "qbittorrent")]
    list.push((
        lab_apis::qbittorrent::META.name,
        lab_apis::qbittorrent::META.required_env,
    ));
    #[cfg(feature = "tailscale")]
    list.push((
        lab_apis::tailscale::META.name,
        lab_apis::tailscale::META.required_env,
    ));
    #[cfg(feature = "linkding")]
    list.push((
        lab_apis::linkding::META.name,
        lab_apis::linkding::META.required_env,
    ));
    #[cfg(feature = "memos")]
    list.push((
        lab_apis::memos::META.name,
        lab_apis::memos::META.required_env,
    ));
    #[cfg(feature = "bytestash")]
    list.push((
        lab_apis::bytestash::META.name,
        lab_apis::bytestash::META.required_env,
    ));
    #[cfg(feature = "paperless")]
    list.push((
        lab_apis::paperless::META.name,
        lab_apis::paperless::META.required_env,
    ));
    #[cfg(feature = "arcane")]
    list.push((
        lab_apis::arcane::META.name,
        lab_apis::arcane::META.required_env,
    ));
    #[cfg(feature = "unraid")]
    list.push((
        lab_apis::unraid::META.name,
        lab_apis::unraid::META.required_env,
    ));
    #[cfg(feature = "unifi")]
    list.push((
        lab_apis::unifi::META.name,
        lab_apis::unifi::META.required_env,
    ));
    #[cfg(feature = "overseerr")]
    list.push((
        lab_apis::overseerr::META.name,
        lab_apis::overseerr::META.required_env,
    ));
    #[cfg(feature = "gotify")]
    list.push((
        lab_apis::gotify::META.name,
        lab_apis::gotify::META.required_env,
    ));
    #[cfg(feature = "openai")]
    list.push((
        lab_apis::openai::META.name,
        lab_apis::openai::META.required_env,
    ));
    #[cfg(feature = "qdrant")]
    list.push((
        lab_apis::qdrant::META.name,
        lab_apis::qdrant::META.required_env,
    ));
    #[cfg(feature = "tei")]
    list.push((lab_apis::tei::META.name, lab_apis::tei::META.required_env));
    #[cfg(feature = "apprise")]
    list.push((
        lab_apis::apprise::META.name,
        lab_apis::apprise::META.required_env,
    ));

    list
}

/// Run the doctor subcommand.
pub fn run(format: OutputFormat) -> Result<ExitCode> {
    let mut findings: Vec<Finding> = Vec::new();

    for (service_name, required_env) in service_env_checks() {
        for env in required_env {
            let present = std::env::var(env.name).is_ok_and(|v| !v.is_empty());
            findings.push(Finding {
                service: service_name.into(),
                check: format!("env:{}", env.name),
                severity: if present {
                    Severity::Ok
                } else {
                    Severity::Fail
                },
                message: if present {
                    format!("{} is set", env.name)
                } else {
                    format!("{} is missing ({})", env.name, env.description)
                },
            });
        }
    }

    let report = Report { findings };
    print(&report, format)?;

    let worst = report
        .findings
        .iter()
        .map(|f| f.severity)
        .fold(Severity::Ok, |acc, s| match (acc, s) {
            (Severity::Fail, _) | (_, Severity::Fail) => Severity::Fail,
            (Severity::Warn, _) | (_, Severity::Warn) => Severity::Warn,
            _ => Severity::Ok,
        });

    Ok(match worst {
        Severity::Ok => ExitCode::SUCCESS,
        Severity::Warn => ExitCode::from(1),
        Severity::Fail => ExitCode::from(2),
    })
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
    fn radarr_in_checks_when_feature_enabled() {
        let checks = service_env_checks();
        #[cfg(feature = "radarr")]
        assert!(checks.iter().any(|(name, _)| *name == "radarr"));
        #[cfg(not(feature = "radarr"))]
        assert!(!checks.iter().any(|(name, _)| *name == "radarr"));
    }
}
