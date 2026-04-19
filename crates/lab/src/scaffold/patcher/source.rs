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
    let module_decl = format!("pub mod {name};\n");
    let module_marker = "// [lab-scaffold: cli-modules]\n";
    let content = if content.contains(&module_decl) {
        content.to_string()
    } else {
        insert_once(
            content,
            module_marker,
            &format!("#[cfg(feature = \"{name}\")]\n{module_decl}{module_marker}"),
        )?
    };

    // Anchor 2: enum variant — insert before the stable marker comment.
    let service_pascal = pascal_case(name);
    let service_variant = format!(
        "    #[cfg(feature = \"{name}\")]\n    {service_pascal}({name}::{service_pascal}Args),\n",
    );
    let variants_marker = "    // [lab-scaffold: cli-variants]\n";
    let content = if content.contains(&service_variant) {
        content
    } else {
        insert_once(
            &content,
            variants_marker,
            &format!("{service_variant}{variants_marker}"),
        )?
    };

    // Anchor 3: dispatch arm — insert before the stable marker comment.
    let dispatch_arm = format!(
        "        #[cfg(feature = \"{name}\")]\n        Command::{service_pascal}(args) => {name}::run(args, format).await,\n",
    );
    let dispatch_marker = "        // [lab-scaffold: cli-dispatch]\n";
    if content.contains(&dispatch_arm) {
        Ok(content)
    } else {
        insert_once(
            &content,
            dispatch_marker,
            &format!("{dispatch_arm}{dispatch_marker}"),
        )
    }
}

pub fn patch_mcp_services_rs(name: &str, content: &str) -> Result<String> {
    let module_decl = format!("pub mod {name};\n");
    if content.contains(&module_decl) {
        return Ok(content.to_string());
    }
    let marker = "// [lab-scaffold: mcp-services]\n";
    insert_once(
        content,
        marker,
        &format!("#[cfg(feature = \"{name}\")]\n{module_decl}{marker}"),
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
    let insert = format!("        mount_if_enabled!(v1, state, \"{name}\", \"{name}\", {name});\n");
    if content.contains(&insert) {
        return Ok(content.to_string());
    }
    let marker = "        // [lab-scaffold: api-routes]\n";
    insert_once(content, marker, &format!("{insert}{marker}"))
}

pub fn patch_dispatch_clients_rs(name: &str, content: &str) -> Result<String> {
    // Anchor 1: struct field — insert before the stable marker.
    let service_type = pascal_case(name);
    let field = format!(
        "    #[cfg(feature = \"{name}\")]\n    pub {name}: Option<Arc<lab_apis::{name}::{service_type}Client>>,\n",
    );
    let fields_marker = "    // [lab-scaffold: state-fields]\n";
    let content = if content.contains(&field) {
        content.to_string()
    } else {
        insert_once(content, fields_marker, &format!("{field}{fields_marker}"))?
    };

    // Anchor 2: from_env initializer — insert before the stable marker.
    let load = format!(
        "            #[cfg(feature = \"{name}\")]\n            {name}: crate::dispatch::{name}::client_from_env().map(Arc::new),\n"
    );
    let from_env_marker = "            // [lab-scaffold: state-from-env]\n";
    if content.contains(&load) {
        Ok(content)
    } else {
        insert_once(&content, from_env_marker, &format!("{load}{from_env_marker}"))
    }
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
