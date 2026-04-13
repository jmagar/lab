//! Coverage doc checks.

use std::io::ErrorKind;
use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let path = repo_root.join(format!("docs/coverage/{name}.md"));
    let result = match std::fs::read_to_string(&path) {
        Ok(content) if !content.trim().is_empty() => CheckResult::Pass,
        Ok(_) => CheckResult::Fail(format!("{} is empty", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            CheckResult::Fail(format!("missing {}", path.display()))
        }
        Err(err) => CheckResult::Fail(format!("failed to read {}: {err}", path.display())),
    };
    vec![("docs.coverage".into(), result)]
}
