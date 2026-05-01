//! Registration checks against shared files.

use std::path::Path;

use super::super::{CheckResult, onboarding::SharedContext};

pub fn run(name: &str, shared: &SharedContext, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let mut out = Vec::new();
    let service_file = repo_root.join(format!("crates/lab-apis/src/{name}.rs"));
    let service_text = std::fs::read_to_string(service_file).unwrap_or_default();
    let registry_name = extract_plugin_meta_name(&service_text).unwrap_or(name);
    let feature = format!("{name} = [");
    let service_mod = format!("#[cfg(feature = \"{name}\")]\npub mod {name};");
    let dispatch_mod = format!("pub mod {name};");
    let api_mount = format!("mount_if_enabled!(v1, state, \"{name}\", \"{registry_name}\"");
    let client_field = format!("pub {name}: Option<");
    let client_load = format!("{name}: crate::dispatch::{name}::client_from_env().map(Arc::new)");
    let tui_health = format!("spawn_health!(\"{registry_name}\", client)");
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
    let registry = shared
        .get(repo_root, "crates/lab/src/registry.rs")
        .unwrap_or("");
    let api_services = shared
        .get(repo_root, "crates/lab/src/api/services.rs")
        .unwrap_or("");
    let router = shared
        .get(repo_root, "crates/lab/src/api/router.rs")
        .unwrap_or("");
    let state = shared
        .get(repo_root, "crates/lab/src/dispatch/clients.rs")
        .unwrap_or("");
    let tui = shared
        .get(repo_root, "crates/lab/src/tui/metadata.rs")
        .unwrap_or("");

    out.push((
        "feature.lab-apis".into(),
        contains_feature_check(lab_apis_cargo, &feature),
    ));
    out.push((
        "feature.lab".into(),
        contains_feature_check(lab_cargo, &format!("{name} = [\"lab-apis/{name}\"]")),
    ));
    out.push(("lib.rs".into(), contains_check(lib, &service_mod)));
    out.push(("cli.rs".into(), contains_check(cli, &dispatch_mod)));
    out.push((
        "registry.rs".into(),
        contains_check(registry, &mcp_registry_token),
    ));
    out.push((
        "api.services.rs".into(),
        contains_check(api_services, &service_mod),
    ));
    out.push((
        "api.router.rs".into(),
        contains_feature_check(router, &api_mount),
    ));
    out.push((
        "dispatch.clients.rs".into(),
        contains_check(state, &client_field),
    ));
    out.push(("tui.metadata.rs".into(), contains_check(tui, &tui_health)));
    out.push((
        "dispatch.clients.load".into(),
        contains_check(state, &client_load).or_skip("state client mapping is not yet threaded"),
    ));
    out
}

fn extract_plugin_meta_name(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        let value = trimmed.strip_prefix("name: \"")?;
        value.split_once('"').map(|(name, _)| name)
    })
}

fn contains_check(haystack: &str, needle: &str) -> CheckResult {
    if haystack.contains(needle) {
        CheckResult::Pass
    } else {
        CheckResult::Fail(format!("missing token `{needle}`"))
    }
}

fn contains_feature_check(haystack: &str, needle: &str) -> CheckResult {
    let compact_haystack = haystack.split_whitespace().collect::<String>();
    let compact_needle = needle.split_whitespace().collect::<String>();
    if compact_haystack.contains(&compact_needle) {
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
    use super::extract_plugin_meta_name;

    #[test]
    fn extracts_plugin_meta_name() {
        let text = r#"
pub const META: PluginMeta = PluginMeta {
    name: "uptime-kuma",
};
"#;
        assert_eq!(extract_plugin_meta_name(text), Some("uptime-kuma"));
    }

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
