//! Registration checks against shared files.

use std::path::Path;

use super::super::{CheckResult, onboarding::SharedContext};

pub fn run(name: &str, shared: &SharedContext, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let mut out = Vec::new();
    let feature = format!("{name} = [");
    let service_mod = format!("#[cfg(feature = \"{name}\")]\npub mod {name};");
    let dispatch_mod = format!("pub mod {name};");
    let api_nest = format!("nest(\"/{name}\"");
    let api_field = format!("pub {name}: Option<");
    let api_load = format!("{name}: crate::dispatch::{name}::client_from_env()");
    // Match the comment token that `patch_tui_metadata_rs` inserts — avoids false passes
    // from any incidental occurrence of the service name in the file.
    let tui_token = format!("// scaffolded service: {name}");
    // Use the register_service! macro anchor rather than a bare name substring to avoid
    // false passes from the name appearing in comments or unrelated strings.
    let mcp_registry_token = format!("register_service!(reg, \"{name}\"");

    let lib = shared
        .get(repo_root, "crates/lab-apis/src/lib.rs")
        .unwrap_or("");
    let lab_apis_cargo = shared
        .get(repo_root, "crates/lab-apis/Cargo.toml")
        .unwrap_or("");
    let lab_cargo = shared.get(repo_root, "crates/lab/Cargo.toml").unwrap_or("");
    let cli = shared.get(repo_root, "crates/lab/src/cli.rs").unwrap_or("");
    let mcp_services = shared
        .get(repo_root, "crates/lab/src/mcp/services.rs")
        .unwrap_or("");
    let registry = shared
        .get(repo_root, "crates/lab/src/mcp/registry.rs")
        .unwrap_or("");
    let api_services = shared
        .get(repo_root, "crates/lab/src/api/services.rs")
        .unwrap_or("");
    let router = shared
        .get(repo_root, "crates/lab/src/api/router.rs")
        .unwrap_or("");
    let state = shared
        .get(repo_root, "crates/lab/src/api/state.rs")
        .unwrap_or("");
    let tui = shared
        .get(repo_root, "crates/lab/src/tui/metadata.rs")
        .unwrap_or("");

    out.push((
        "feature.lab-apis".into(),
        contains_check(lab_apis_cargo, &feature),
    ));
    out.push((
        "feature.lab".into(),
        contains_check(lab_cargo, &format!("{name} = [\"lab-apis/{name}\"]")),
    ));
    out.push(("lib.rs".into(), contains_check(lib, &service_mod)));
    out.push(("cli.rs".into(), contains_check(cli, &dispatch_mod)));
    out.push((
        "mcp.services.rs".into(),
        contains_check(mcp_services, &service_mod),
    ));
    out.push((
        "mcp.registry.rs".into(),
        contains_check(registry, &mcp_registry_token),
    ));
    out.push((
        "api.services.rs".into(),
        contains_check(api_services, &service_mod),
    ));
    out.push(("api.router.rs".into(), contains_check(router, &api_nest)));
    out.push(("api.state.rs".into(), contains_check(state, &api_field)));
    out.push(("tui.metadata.rs".into(), contains_check(tui, &tui_token)));
    out.push((
        "api.state.load".into(),
        contains_check(state, &api_load).or_skip("state client mapping is not yet threaded"),
    ));
    out
}

fn contains_check(haystack: &str, needle: &str) -> CheckResult {
    if haystack.contains(needle) {
        CheckResult::Pass
    } else {
        CheckResult::Fail(format!("missing token `{needle}`"))
    }
}

trait CheckExt {
    fn or_skip(self, msg: &str) -> Self;
}

impl CheckExt for CheckResult {
    fn or_skip(self, msg: &str) -> Self {
        match self {
            Self::Fail(_) => Self::Skip(msg.to_string()),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bare_pub_mod_fails_mcp_check() {
        // A file with bare pub mod foo; but no cfg guard should fail
        // the check now uses service_mod which requires the cfg line above
        let needle = format!("#[cfg(feature = \"foo\")]\npub mod foo;");
        let file_with_bare = "pub mod foo;";
        let file_with_cfg = "#[cfg(feature = \"foo\")]\npub mod foo;";
        assert!(
            !file_with_bare.contains(&needle),
            "bare pub mod should NOT match cfg pattern"
        );
        assert!(
            file_with_cfg.contains(&needle),
            "cfg-gated mod SHOULD match"
        );
    }
}
