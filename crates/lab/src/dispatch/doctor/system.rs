//! Local system probes for `system.checks`.
//!
//! All file and env I/O lives here, never in `lab-apis`.

use super::types::{Finding, Severity, service_env_checks};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn path_check(service: &str, label: &str, path: &str, severity_on_missing: Severity) -> Finding {
    let exists = std::path::Path::new(path).exists();
    Finding {
        service: service.to_string(),
        check: label.to_string(),
        severity: if exists { Severity::Ok } else { severity_on_missing },
        message: if exists {
            format!("{path} found")
        } else {
            format!("{path} not found")
        },
    }
}

fn command_check(service: &str, label: &str, cmd: &str) -> Finding {
    let found = std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    Finding {
        service: service.to_string(),
        check: label.to_string(),
        severity: if found { Severity::Ok } else { Severity::Warn },
        message: if found {
            format!("`{cmd}` is available")
        } else {
            format!("`{cmd}` not found on PATH")
        },
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Run all local system probes: env-var checks, config files, Docker, disk.
///
/// Order: env-var checks first (preserves current `lab doctor` output), then
/// system-level checks.
pub fn run_system_checks() -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    // --- Env var checks (current lab doctor behaviour; preserved for output parity) ---
    for (service_name, required_env) in service_env_checks() {
        for env in required_env {
            let present = std::env::var(env.name).is_ok_and(|v| !v.is_empty());
            findings.push(Finding {
                service: service_name.into(),
                check: format!("env:{}", env.name),
                severity: if present { Severity::Ok } else { Severity::Fail },
                message: if present {
                    format!("{} is set", env.name)
                } else {
                    format!("{} is missing ({})", env.name, env.description)
                },
            });
        }
    }

    // --- Lab config files ---
    let home = std::env::var("HOME").unwrap_or_default();
    findings.push(path_check(
        "lab",
        "config:~/.lab/.env",
        &format!("{home}/.lab/.env"),
        Severity::Warn,
    ));
    findings.push(path_check(
        "lab",
        "config:~/.lab/config.toml",
        &format!("{home}/.lab/config.toml"),
        Severity::Warn,
    ));

    // --- AI assistant configs (informational) ---
    for (name, rel_path) in [(".claude", "claude"), (".codex", "codex"), (".gemini", "gemini")] {
        let full = format!("{home}/{name}");
        let exists = std::path::Path::new(&full).exists();
        findings.push(Finding {
            service: "lab".into(),
            check: format!("config:~/{name}"),
            severity: Severity::Ok,
            message: if exists {
                format!("~/{name} present ({rel_path} detected)")
            } else {
                format!("~/{name} not present")
            },
        });
    }

    // --- Docker ---
    findings.push(path_check(
        "system",
        "docker:socket",
        "/var/run/docker.sock",
        Severity::Warn,
    ));
    findings.push(command_check("system", "docker:cli", "docker"));
    findings.push(command_check("system", "docker:compose-plugin", "docker"));

    // --- Rust toolchain ---
    findings.push(command_check("system", "rust:cargo", "cargo"));

    // --- Disk space: warn when / exceeds 90 % used ---
    disk_check(&mut findings);

    findings
}

#[cfg(target_os = "linux")]
fn disk_check(findings: &mut Vec<Finding>) {
    let Ok(output) = std::process::Command::new("df")
        .args(["-h", "--output=pcent", "/"])
        .output()
    else {
        return;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pct: Option<u64> = stdout
        .lines()
        .nth(1)
        .and_then(|l| l.trim().trim_end_matches('%').parse().ok());
    if let Some(used) = pct {
        findings.push(Finding {
            service: "system".into(),
            check: "disk:/".into(),
            severity: if used >= 90 { Severity::Warn } else { Severity::Ok },
            message: format!("/ is {used}% used"),
        });
    }
}

#[cfg(not(target_os = "linux"))]
fn disk_check(_findings: &mut Vec<Finding>) {}
