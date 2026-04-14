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
    let content = insert_once(
        content,
        "pub mod serve;\n",
        &format!("pub mod serve;\npub mod {name};\n"),
    )?;
    let service_variant = format!(
        "    #[cfg(feature = \"{name}\")]\n    {service}({snake}::{service}Args),\n",
        service = pascal_case(name),
        snake = name,
    );
    let content = insert_once(
        &content,
        "    #[cfg(feature = \"apprise\")]\n    Apprise(apprise::AppriseArgs),\n",
        &format!(
            "{service_variant}    #[cfg(feature = \"apprise\")]\n    Apprise(apprise::AppriseArgs),\n"
        ),
    )?;

    let dispatch_arm = format!(
        "        #[cfg(feature = \"{name}\")]\n        Command::{service}(args) => {snake}::run(args, format).await,\n",
        service = pascal_case(name),
        snake = name,
    );
    insert_once(
        &content,
        "        #[cfg(feature = \"apprise\")]\n        Command::Apprise(args) => apprise::run(args, format).await,\n",
        &format!(
            "{dispatch_arm}        #[cfg(feature = \"apprise\")]\n        Command::Apprise(args) => apprise::run(args, format).await,\n"
        ),
    )
}

#[allow(clippy::unnecessary_wraps)]
pub fn patch_mcp_services_rs(name: &str, content: &str) -> Result<String> {
    Ok(insert_before_eof(
        content,
        &format!("\n#[cfg(feature = \"{name}\")]\npub mod {name};\n"),
    ))
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
    let insert = format!(
        "    #[cfg(feature = \"{name}\")]\n    if state.registry.services().iter().any(|s| s.name == \"{name}\") {{\n        v1 = v1.nest(\"/{name}\", services::{name}::routes(state.clone()));\n    }}\n"
    );
    insert_once(
        content,
        "    let x_request_id = HeaderName::from_static(\"x-request-id\");",
        &format!("{insert}    let x_request_id = HeaderName::from_static(\"x-request-id\");"),
    )
}

pub fn patch_dispatch_clients_rs(name: &str, content: &str) -> Result<String> {
    let field = format!(
        "    #[cfg(feature = \"{name}\")]\n    pub {name}: Option<Arc<lab_apis::{name}::{service_type}Client>>,",
        service_type = pascal_case(name)
    );
    let content = insert_once(
        content,
        "    #[cfg(feature = \"prowlarr\")]\n    pub prowlarr: Option<Arc<lab_apis::prowlarr::ProwlarrClient>>,",
        &format!(
            "    #[cfg(feature = \"prowlarr\")]\n    pub prowlarr: Option<Arc<lab_apis::prowlarr::ProwlarrClient>>,\n{field}"
        ),
    )?;

    let load = format!(
        "            #[cfg(feature = \"{name}\")]\n            {name}: crate::dispatch::{name}::client_from_env().map(Arc::new),"
    );
    insert_once(
        &content,
        "            #[cfg(feature = \"prowlarr\")]\n            prowlarr: crate::dispatch::prowlarr::client_from_env().map(Arc::new),",
        &format!(
            "            #[cfg(feature = \"prowlarr\")]\n            prowlarr: crate::dispatch::prowlarr::client_from_env().map(Arc::new),\n{load}"
        ),
    )
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
