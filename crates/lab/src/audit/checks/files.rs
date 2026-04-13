//! File existence checks.

use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_'))
    {
        return vec![(
            "service".to_string(),
            CheckResult::Fail(format!("invalid service name `{name}`")),
        )];
    }

    let mut out = Vec::new();
    for rel in [
        format!("crates/lab-apis/src/{name}.rs"),
        format!("crates/lab-apis/src/{name}/client.rs"),
        format!("crates/lab-apis/src/{name}/types.rs"),
        format!("crates/lab-apis/src/{name}/error.rs"),
        format!("crates/lab/src/dispatch/{name}.rs"),
        format!("crates/lab/src/dispatch/{name}/catalog.rs"),
        format!("crates/lab/src/dispatch/{name}/client.rs"),
        format!("crates/lab/src/dispatch/{name}/dispatch.rs"),
        format!("crates/lab/src/dispatch/{name}/params.rs"),
        format!("crates/lab/src/cli/{name}.rs"),
        format!("crates/lab/src/mcp/services/{name}.rs"),
        format!("crates/lab/src/api/services/{name}.rs"),
    ] {
        let path = repo_root.join(&rel);
        out.push((
            rel.clone(),
            if path.exists() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("missing {rel}"))
            },
        ));
    }
    out
}
