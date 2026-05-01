//! Audit orchestration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::checks;
use super::{AuditReport, ServiceReport};

/// Shared files loaded once for cross-service checks.
#[derive(Debug, Clone, Default)]
pub struct SharedContext {
    pub files: HashMap<PathBuf, String>,
}

impl SharedContext {
    pub fn load(repo_root: &Path) -> std::io::Result<Self> {
        let mut files = HashMap::new();
        for rel in [
            "crates/lab-apis/Cargo.toml",
            "crates/lab/Cargo.toml",
            "crates/lab-apis/src/lib.rs",
            "crates/lab/src/cli.rs",
            "crates/lab/src/mcp/services.rs",
            "crates/lab/src/registry.rs",
            "crates/lab/src/dispatch/clients.rs",
            "crates/lab/src/api/services.rs",
            "crates/lab/src/api/router.rs",
            "crates/lab/src/tui/metadata.rs",
        ] {
            let path = repo_root.join(rel);
            files.insert(path.clone(), std::fs::read_to_string(path)?);
        }
        Ok(Self { files })
    }

    pub(crate) fn get(&self, repo_root: &Path, rel: &str) -> Option<&str> {
        self.files.get(&repo_root.join(rel)).map(String::as_str)
    }
}

pub fn audit_service(name: &str, shared: &SharedContext, repo_root: &Path) -> ServiceReport {
    let mut checks_out = Vec::new();
    checks_out.extend(checks::files::run(name, repo_root));
    checks_out.extend(checks::registration::run(name, shared, repo_root));
    checks_out.extend(checks::ui_schema::run(name, repo_root));
    checks_out.extend(checks::dispatch::run(name, repo_root));
    checks_out.extend(checks::tests::run(name, repo_root));
    checks_out.extend(checks::docs::run(name, repo_root));
    ServiceReport {
        service: name.to_string(),
        checks: checks_out,
    }
}

pub fn audit_services(names: &[String], repo_root: &Path) -> AuditReport {
    let shared = SharedContext::load(repo_root).unwrap_or_else(|err| {
        tracing::warn!(error = %err, "failed to load shared context; registration checks may show false failures");
        SharedContext::default()
    });
    let services = names
        .iter()
        .map(|name| audit_service(name, &shared, repo_root))
        .collect();
    AuditReport { services }
}
