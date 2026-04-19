//! Rust source patchers for scaffold generation.

use super::super::service::{Result, ScaffoldError};
use crate::scaffold::templates::pascal_case;

#[allow(clippy::unnecessary_wraps)]
pub fn patch_lib_rs(name: &str, content: &str) -> Result<String> {
    Ok(insert_before_eof(
        content,
        &format!("#[cfg(feature = \"{name}\")]\npub mod {name};\n"),
    ))
}

pub fn patch_dispatch_rs(name: &str, content: &str) -> Result<String> {
    insert_once(
        content,
        "pub mod helpers;\n",
        &format!("pub mod helpers;\n#[cfg(feature = \"{name}\")]\npub mod {name};\n"),
    )
}

pub fn patch_cli_rs(name: &str, content: &str) -> Result<String> {
    // Anchor 1: module declaration — insert before the stable marker comment.
    let module_marker = "// [lab-scaffold: cli-modules]\n";
    let content = insert_once(
        content,
        module_marker,
        &format!("#[cfg(feature = \"{name}\")]\npub mod {name};\n{module_marker}"),
    )?;

    // Anchor 2: enum variant — insert before the stable marker comment.
    let service_variant = format!(
        "    #[cfg(feature = \"{name}\")]\n    {service}({snake}::{service}Args),\n",
        service = pascal_case(name),
        snake = name,
    );
    let variants_marker = "    // [lab-scaffold: cli-variants]\n";
    let content = insert_once(
        &content,
        variants_marker,
        &format!("{service_variant}{variants_marker}"),
    )?;

    // Anchor 3: dispatch arm — insert before the stable marker comment.
    let dispatch_arm = format!(
        "        #[cfg(feature = \"{name}\")]\n        Command::{service}(args) => {snake}::run(args, format).await,\n",
        service = pascal_case(name),
        snake = name,
    );
    let dispatch_marker = "        // [lab-scaffold: cli-dispatch]\n";
    insert_once(
        &content,
        dispatch_marker,
        &format!("{dispatch_arm}{dispatch_marker}"),
    )
}

pub fn patch_mcp_services_rs(name: &str, content: &str) -> Result<String> {
    // Anchor on the stable marker so refactoring doesn't silently fall through.
    let marker = "// [lab-scaffold: mcp-services]\n";
    insert_once(
        content,
        marker,
        &format!("#[cfg(feature = \"{name}\")]\npub mod {name};\n{marker}"),
    )
}

pub fn patch_mcp_registry_rs(name: &str, content: &str) -> Result<String> {
    insert_once(
        content,
        "    reg\n}",
        &format!("    register_service!(reg, \"{name}\", {name});\n\n    reg\n}}"),
    )
}

#[allow(clippy::unnecessary_wraps)]
pub fn patch_api_services_rs(name: &str, content: &str) -> Result<String> {
    Ok(insert_before_eof(
        content,
        &format!("\n#[cfg(feature = \"{name}\")]\npub mod {name};\n"),
    ))
}

pub fn patch_api_router_rs(name: &str, content: &str) -> Result<String> {
    // Anchor on the stable marker instead of the HeaderName line.
    let marker = "        // [lab-scaffold: api-routes]\n";
    let insert = format!("        mount_if_enabled!(v1, state, \"{name}\", \"{name}\", {name});\n");
    insert_once(content, marker, &format!("{insert}{marker}"))
}

pub fn patch_dispatch_clients_rs(name: &str, content: &str) -> Result<String> {
    // Anchor 1: struct field — insert before the stable marker.
    let fields_marker = "    // [lab-scaffold: state-fields]\n";
    let field = format!(
        "    #[cfg(feature = \"{name}\")]\n    pub {name}: Option<Arc<lab_apis::{name}::{service_type}Client>>,\n",
        service_type = pascal_case(name)
    );
    let content = insert_once(content, fields_marker, &format!("{field}{fields_marker}"))?;

    // Anchor 2: from_env initializer — insert before the stable marker.
    let from_env_marker = "            // [lab-scaffold: state-from-env]\n";
    let load = format!(
        "            #[cfg(feature = \"{name}\")]\n            {name}: crate::dispatch::{name}::client_from_env().map(Arc::new),\n"
    );
    insert_once(&content, from_env_marker, &format!("{load}{from_env_marker}"))
}

fn insert_before_eof(content: &str, insert: &str) -> String {
    if content.contains(insert) {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len() + insert.len());
    out.push_str(content);
    out.push_str(insert);
    out
}

fn insert_once(content: &str, needle: &str, replacement: &str) -> Result<String> {
    if content.contains(replacement) {
        return Ok(content.to_string());
    }
    let Some(idx) = content.find(needle) else {
        return Err(ScaffoldError::PatchAnchorNotFound {
            anchor: needle.to_string(),
        });
    };
    let mut out = String::with_capacity(content.len() + replacement.len() - needle.len());
    out.push_str(&content[..idx]);
    out.push_str(replacement);
    out.push_str(&content[idx + needle.len()..]);
    Ok(out)
}
