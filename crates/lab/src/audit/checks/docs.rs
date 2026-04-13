//! Coverage doc checks.

use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let path = repo_root.join(format!("docs/coverage/{name}.md"));
    let result = match std::fs::read_to_string(&path) {
        Ok(content) if !content.trim().is_empty() => CheckResult::Pass,
        Ok(_) => CheckResult::Fail(format!("{} is empty", path.display())),
        Err(_) => CheckResult::Fail(format!("missing {}", path.display())),
    };
    vec![("docs.coverage".into(), result)]
}
