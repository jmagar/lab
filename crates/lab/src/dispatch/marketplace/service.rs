#![allow(dead_code)]

use lab_apis::marketplace::{Artifact, Marketplace, MarketplaceRuntime, Plugin, PluginComponent};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::marketplace::backend::{MarketplaceBackend, PluginFilter};
use crate::dispatch::marketplace::backends::claude::ClaudeMarketplaceBackend;
use crate::dispatch::marketplace::backends::codex::CodexMarketplaceBackend;
use crate::dispatch::marketplace::runtime::{parse_marketplace_runtime, runtime_display_name};

pub fn runtime_from_params(params: &Value) -> Result<Option<MarketplaceRuntime>, ToolError> {
    params
        .get("runtime")
        .and_then(Value::as_str)
        .map(parse_marketplace_runtime)
        .transpose()
}

fn claude_backend() -> ClaudeMarketplaceBackend {
    ClaudeMarketplaceBackend
}

fn codex_backend() -> CodexMarketplaceBackend {
    CodexMarketplaceBackend
}

pub fn list_plugins_sync(
    runtime: Option<MarketplaceRuntime>,
    filter: Option<String>,
) -> Result<Vec<Plugin>, ToolError> {
    let filter = PluginFilter {
        marketplace: filter,
    };
    match runtime {
        Some(MarketplaceRuntime::Claude) => claude_backend().list_plugins(filter),
        Some(MarketplaceRuntime::Codex) => codex_backend().list_plugins(filter),
        Some(MarketplaceRuntime::Gemini) => Ok(Vec::new()),
        None => {
            let mut out = Vec::new();
            let claude = claude_backend();
            if claude.is_available() {
                out.extend(claude.list_plugins(filter.clone())?);
            }
            let codex = codex_backend();
            if codex.is_available() {
                out.extend(codex.list_plugins(filter)?);
            }
            Ok(out)
        }
    }
}

pub async fn sources_list(
    runtime: Option<MarketplaceRuntime>,
) -> Result<Vec<Marketplace>, ToolError> {
    tokio::task::spawn_blocking(move || match runtime {
        Some(MarketplaceRuntime::Claude) => claude_backend().list_sources(),
        Some(MarketplaceRuntime::Codex) => codex_backend().list_sources(),
        Some(MarketplaceRuntime::Gemini) => Ok(Vec::new()),
        None => {
            let mut out = Vec::new();
            let claude = claude_backend();
            if claude.is_available() {
                out.extend(claude.list_sources()?);
            }
            let codex = codex_backend();
            if codex.is_available() {
                out.extend(codex.list_sources()?);
            }
            Ok(out)
        }
    })
    .await
    .map_err(join_err)?
}

pub async fn plugins_list(
    runtime: Option<MarketplaceRuntime>,
    filter: Option<String>,
) -> Result<Vec<Plugin>, ToolError> {
    tokio::task::spawn_blocking(move || list_plugins_sync(runtime, filter))
        .await
        .map_err(join_err)?
}

pub async fn plugin_get(
    runtime: Option<MarketplaceRuntime>,
    id: &str,
) -> Result<Plugin, ToolError> {
    let id = id.to_string();
    tokio::task::spawn_blocking(move || get_plugin_sync(runtime, &id))
        .await
        .map_err(join_err)?
}

pub async fn plugin_artifacts(
    runtime: Option<MarketplaceRuntime>,
    id: &str,
) -> Result<Vec<Artifact>, ToolError> {
    let id = id.to_string();
    tokio::task::spawn_blocking(move || match runtime {
        Some(MarketplaceRuntime::Claude) => claude_backend().list_artifacts(&id),
        Some(MarketplaceRuntime::Codex) => codex_backend().list_artifacts(&id),
        Some(MarketplaceRuntime::Gemini) => Err(unsupported_runtime_action(
            MarketplaceRuntime::Gemini,
            "plugin.artifacts",
        )),
        None => {
            if let Ok(plugin) = get_plugin_sync(None, &id) {
                match plugin.runtime {
                    Some(MarketplaceRuntime::Claude) => claude_backend().list_artifacts(&id),
                    Some(MarketplaceRuntime::Codex) => codex_backend().list_artifacts(&id),
                    Some(MarketplaceRuntime::Gemini) => Err(unsupported_runtime_action(
                        MarketplaceRuntime::Gemini,
                        "plugin.artifacts",
                    )),
                    None => Err(ToolError::Sdk {
                        sdk_kind: "not_found".into(),
                        message: format!("plugin `{id}` not found"),
                    }),
                }
            } else {
                Err(ToolError::Sdk {
                    sdk_kind: "not_found".into(),
                    message: format!("plugin `{id}` not found"),
                })
            }
        }
    })
    .await
    .map_err(join_err)?
}

pub async fn plugin_components(
    runtime: Option<MarketplaceRuntime>,
    id: &str,
) -> Result<Vec<PluginComponent>, ToolError> {
    let id = id.to_string();
    tokio::task::spawn_blocking(move || match runtime {
        Some(MarketplaceRuntime::Claude) => claude_backend().list_components(&id),
        Some(MarketplaceRuntime::Codex) => codex_backend().list_components(&id),
        Some(MarketplaceRuntime::Gemini) => Ok(Vec::new()),
        None => {
            let plugin = get_plugin_sync(None, &id)?;
            match plugin.runtime {
                Some(MarketplaceRuntime::Claude) => claude_backend().list_components(&id),
                Some(MarketplaceRuntime::Codex) => codex_backend().list_components(&id),
                Some(MarketplaceRuntime::Gemini) | None => Ok(Vec::new()),
            }
        }
    })
    .await
    .map_err(join_err)?
}

pub fn require_claude_write(
    runtime: Option<MarketplaceRuntime>,
    action: &str,
) -> Result<(), ToolError> {
    match runtime {
        Some(MarketplaceRuntime::Codex) => Err(unsupported_runtime_action(
            MarketplaceRuntime::Codex,
            action,
        )),
        Some(MarketplaceRuntime::Gemini) => Err(unsupported_runtime_action(
            MarketplaceRuntime::Gemini,
            action,
        )),
        _ => Ok(()),
    }
}

fn get_plugin_sync(runtime: Option<MarketplaceRuntime>, id: &str) -> Result<Plugin, ToolError> {
    match runtime {
        Some(MarketplaceRuntime::Claude) => claude_backend().get_plugin(id),
        Some(MarketplaceRuntime::Codex) => codex_backend().get_plugin(id),
        Some(MarketplaceRuntime::Gemini) => Err(ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("plugin `{id}` not found"),
        }),
        None => {
            let mut matches = Vec::new();
            let claude = claude_backend();
            if claude.is_available() {
                if let Ok(plugin) = claude.get_plugin(id) {
                    matches.push(plugin);
                }
            }
            let codex = codex_backend();
            if codex.is_available() {
                if let Ok(plugin) = codex.get_plugin(id) {
                    matches.push(plugin);
                }
            }
            match matches.len() {
                0 => Err(ToolError::Sdk {
                    sdk_kind: "not_found".into(),
                    message: format!("plugin `{id}` not found"),
                }),
                1 => Ok(matches.remove(0)),
                _ => Err(ToolError::Conflict {
                    message: format!(
                        "plugin `{id}` exists in multiple runtimes; pass `runtime` explicitly"
                    ),
                    existing_id: id.to_string(),
                }),
            }
        }
    }
}

fn unsupported_runtime_action(runtime: MarketplaceRuntime, action: &str) -> ToolError {
    ToolError::InvalidParam {
        message: format!(
            "action `{action}` is not supported for runtime `{}` in Phase 1",
            runtime_display_name(runtime)
        ),
        param: "runtime".into(),
    }
}

fn join_err(error: tokio::task::JoinError) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("join error: {error}"),
    }
}
