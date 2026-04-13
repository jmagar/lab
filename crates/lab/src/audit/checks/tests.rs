//! Test existence checks.

use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let mut out = Vec::new();
    let sdk = repo_root.join(format!("crates/lab-apis/tests/{name}_client.rs"));
    let dispatch = repo_root.join(format!("crates/lab/src/dispatch/{name}/dispatch.rs"));
    let mcp = repo_root.join(format!("crates/lab/src/mcp/services/{name}.rs"));
    let api = repo_root.join(format!("crates/lab/src/api/services/{name}.rs"));
    out.push(("tests.sdk".into(), file_or_skip(&sdk)));
    out.push(("tests.dispatch".into(), file_or_skip(&dispatch)));
    out.push(("tests.mcp".into(), file_or_skip(&mcp)));
    out.push(("tests.api".into(), file_or_skip(&api)));
    out
}

fn file_or_skip(path: &Path) -> CheckResult {
    if path.exists() {
        CheckResult::Pass
    } else {
        CheckResult::Skip(format!("not yet present: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn file_or_skip_missing_returns_skip() {
        let result = file_or_skip(Path::new("/nonexistent/path/that/cannot/exist"));
        assert!(matches!(result, CheckResult::Skip(_)), "missing file should be Skip, not Fail");
    }
}
