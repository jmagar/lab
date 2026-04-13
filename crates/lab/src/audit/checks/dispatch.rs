//! Dispatch layout checks.

use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let mut out = Vec::new();
    let entry = repo_root.join(format!("crates/lab/src/dispatch/{name}.rs"));
    let client = repo_root.join(format!("crates/lab/src/dispatch/{name}/client.rs"));
    let entry_text = std::fs::read_to_string(entry).unwrap_or_default();
    let client_text = std::fs::read_to_string(client).unwrap_or_default();

    out.push((
        "dispatch.entrypoint".into(),
        contains_all(
            &entry_text,
            &[
                "ACTIONS",
                "client_from_env",
                "dispatch",
            ],
        ),
    ));
    out.push((
        "dispatch.client".into(),
        contains_all(
            &client_text,
            &[
                "client_from_env()",
                "require_client()",
                "not_configured_error()",
            ],
        ),
    ));
    out
}

fn contains_all(haystack: &str, needles: &[&str]) -> CheckResult {
    for needle in needles {
        if !haystack.contains(needle) {
            return CheckResult::Fail(format!("missing token `{needle}`"));
        }
    }
    CheckResult::Pass
}
