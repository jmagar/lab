//! Dispatch for `mcp.*` actions within the marketplace service.
//!
//! These actions were absorbed from `dispatch/mcpregistry/` as part of lab-zxx5.2.
//! The `dispatch/mcpregistry/` directory is retained until lab-zxx5.4.
//!
//! All `mcp.*` routing is feature-gated on `mcpregistry`. When the feature is
//! absent, every `mcp.*` action returns a structured `not_configured` error.

#[cfg(feature = "mcpregistry")]
use lab_apis::mcpregistry::McpRegistryClient;
#[cfg(feature = "mcpregistry")]
use lab_apis::mcpregistry::types::ServerJSON;
use serde_json::Value;

use crate::dispatch::error::ToolError;
#[cfg(feature = "mcpregistry")]
use crate::dispatch::helpers::to_json;
#[cfg(feature = "mcpregistry")]
use crate::dispatch::marketplace::LAB_REGISTRY_META_NAMESPACE;
use crate::dispatch::marketplace::mcp_catalog::MCP_ACTIONS;
use crate::dispatch::marketplace::mcp_client;
#[cfg(feature = "mcpregistry")]
use crate::dispatch::marketplace::mcp_params;

/// Dispatch a `mcp.*` action using a freshly constructed client.
///
/// Called from `marketplace/dispatch.rs` for any action with the `mcp.` prefix.
pub async fn dispatch_mcp(action: &str, params: Value) -> Result<Value, ToolError> {
    #[cfg(feature = "mcpregistry")]
    {
        let client = mcp_client::require_mcp_client()?;
        dispatch_mcp_with_client(&client, action, params).await
    }
    #[cfg(not(feature = "mcpregistry"))]
    {
        let _ = (action, params);
        Err(mcp_client::not_configured_error())
    }
}

/// Dispatch a `mcp.*` action with a pre-built client (used by API handlers with AppState).
#[cfg(feature = "mcpregistry")]
pub async fn dispatch_mcp_with_client(
    client: &McpRegistryClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "mcp.config" => Ok(serde_json::json!({
            "url": mcp_client::configured_registry_url()?
        })),
        "mcp.list" => {
            let p = mcp_params::list_servers_params(&params)?;
            if let Some(param) = ["sort_by", "order"]
                .into_iter()
                .find(|name| params.get(*name).is_some())
            {
                return Err(ToolError::InvalidParam {
                    message: "sort_by/order are not supported on the paginated registry surface"
                        .to_string(),
                    param: param.to_string(),
                });
            }
            to_json(client.list_servers(p).await?)
        }
        "mcp.get" => {
            let name = mcp_params::require_name(&params)?;
            to_json(client.get_server(&name, "latest").await?)
        }
        "mcp.versions" => {
            let name = mcp_params::require_name(&params)?;
            to_json(client.list_versions(&name).await?)
        }
        "mcp.validate" => {
            let server_json: ServerJSON = serde_json::from_value(params["server_json"].clone())
                .map_err(|e| ToolError::Sdk {
                    sdk_kind: "invalid_param".to_string(),
                    message: format!("invalid server_json: {e}"),
                })?;
            to_json(client.validate(&server_json).await?)
        }
        "mcp.install" => dispatch_mcp_install(client, &params).await,
        "mcp.uninstall" => {
            let gateway_name = params["gateway_name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `gateway_name`".to_string(),
                    param: "gateway_name".to_string(),
                })?
                .to_string();

            // Delegate to gateway.remove — pass confirm:true since the caller already confirmed
            // at the mcp.uninstall level (destructive:true is checked by handle_action).
            crate::dispatch::gateway::dispatch(
                "gateway.remove",
                serde_json::json!({ "name": gateway_name, "confirm": true }),
            )
            .await
        }
        "mcp.meta.get" => dispatch_mcp_local(action, params).await,
        "mcp.meta.set" => dispatch_mcp_local(action, params).await,
        "mcp.meta.delete" => dispatch_mcp_local(action, params).await,
        "mcp.sync" => {
            use crate::config;
            let db_path = config::registry_db_path();
            let store = crate::dispatch::marketplace::store::RegistryStore::open(&db_path)
                .await
                .map_err(|e| ToolError::internal_message(format!("registry store open: {e}")))?;
            let count =
                crate::dispatch::marketplace::sync::perform_sync(&store, client, true, "manual")
                    .await?;
            Ok(serde_json::json!({ "synced": count }))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: MCP_ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Handle `mcp.install`: fetch server details and add a gateway for each gateway_id.
///
/// Takes `params.server_name`, `params.gateway_ids` (Vec<String>), optional `env_vars`.
/// For each gateway_id, calls `gateway.add`. Returns Vec of per-gateway results.
#[cfg(feature = "mcpregistry")]
async fn dispatch_mcp_install(
    client: &McpRegistryClient,
    params: &Value,
) -> Result<Value, ToolError> {
    let name = mcp_params::require_name(params)?;
    let version = params["version"].as_str().unwrap_or("latest");

    let gateway_ids: Vec<String> = match params.get("gateway_ids") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Some(_) => {
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: "`gateway_ids` must be an array of strings".to_string(),
            });
        }
        None => {
            return Err(ToolError::MissingParam {
                message: "missing required parameter `gateway_ids`".to_string(),
                param: "gateway_ids".to_string(),
            });
        }
    };

    if gateway_ids.is_empty() {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: "`gateway_ids` must not be empty".to_string(),
        });
    }

    let server_resp = client.get_server(&name, version).await?;
    let server = &server_resp.server;

    let http_url = server.remotes.iter().find_map(|r| r.url.as_deref());

    let mut results = Vec::new();

    for gateway_id in &gateway_ids {
        let spec = if let Some(url) = http_url {
            match install_http(url, gateway_id, params).await {
                Ok(s) => s,
                Err(e) => {
                    results.push(serde_json::json!({
                        "gateway_id": gateway_id,
                        "ok": false,
                        "error": e.to_string(),
                    }));
                    continue;
                }
            }
        } else if let Some(pkg) = server.packages.first() {
            match install_stdio(pkg, gateway_id, params, &name) {
                Ok(s) => s,
                Err(e) => {
                    results.push(serde_json::json!({
                        "gateway_id": gateway_id,
                        "ok": false,
                        "error": e.to_string(),
                    }));
                    continue;
                }
            }
        } else {
            results.push(serde_json::json!({
                "gateway_id": gateway_id,
                "ok": false,
                "error": format!("server '{name}' has no remotes and no packages — cannot install"),
            }));
            continue;
        };

        // Delegate to gateway.add — confirm:true because the caller already confirmed.
        match crate::dispatch::gateway::dispatch(
            "gateway.add",
            serde_json::json!({ "spec": spec, "confirm": true }),
        )
        .await
        {
            Ok(result) => {
                results.push(serde_json::json!({
                    "gateway_id": gateway_id,
                    "ok": true,
                    "result": result,
                }));
            }
            Err(e) => {
                results.push(serde_json::json!({
                    "gateway_id": gateway_id,
                    "ok": false,
                    "error": e.to_string(),
                }));
            }
        }
    }

    Ok(serde_json::json!({ "results": results }))
}

/// Build a gateway spec for an HTTP-transport server.
///
/// Validates the URL against SSRF rules and probes for OAuth support.
#[cfg(feature = "mcpregistry")]
async fn install_http(
    url: &str,
    gateway_name: &str,
    params_value: &Value,
) -> Result<Value, ToolError> {
    let url = url.to_string();

    // SSRF validation (synchronous DNS) — must run in spawn_blocking.
    let url_for_check = url.clone();
    tokio::task::spawn_blocking(move || mcp_params::validate_registry_url(&url_for_check))
        .await
        .map_err(|e| {
            ToolError::internal_message(format!("SSRF validation task panicked: {e}"))
        })??;

    // Probe for OAuth support — non-fatal, install proceeds without OAuth on failure.
    let discovered_oauth: Option<Value> =
        if let Some(manager) = crate::dispatch::gateway::current_gateway_manager() {
            match manager.probe_upstream_oauth(&url).await {
                Ok(probe) if probe.oauth_discovered => manager
                    .upstream_oauth_manager(&probe.upstream)
                    .and_then(|m| serde_json::to_value(m.upstream_config().oauth.clone()).ok()),
                Ok(_) | Err(_) => None,
            }
        } else {
            None
        };

    let bearer_token_env = params_value["bearer_token_env"].as_str();

    let mut spec = serde_json::json!({
        "name": gateway_name,
        "url": url,
        "bearer_token_env": bearer_token_env,
        "command": null,
        "args": [],
        "proxy_resources": false,
        "expose_tools": null,
    });

    if let Some(oauth) = discovered_oauth {
        spec["oauth"] = oauth;
    }

    Ok(spec)
}

/// Build a gateway spec for a stdio-transport server, validate security constraints,
/// and write any user-supplied env vars into `~/.lab/.env`.
#[cfg(feature = "mcpregistry")]
fn install_stdio(
    pkg: &lab_apis::mcpregistry::types::Package,
    gateway_name: &str,
    params_value: &Value,
    server_name: &str,
) -> Result<Value, ToolError> {
    // runtimeHint is required for stdio packages.
    let hint = pkg.runtime_hint.as_deref().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "invalid_param".to_string(),
        message: format!(
            "server '{server_name}' package has no runtimeHint — cannot build stdio command"
        ),
    })?;

    mcp_params::validate_runtime_hint(hint)?;

    // Build argv: runtimeArguments + identifier + packageArguments.
    let mut argv: Vec<String> = pkg
        .runtime_arguments
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    argv.push(pkg.identifier.clone());
    argv.extend(
        pkg.package_arguments
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string)),
    );

    mcp_params::validate_stdio_argv(&argv)?;

    // Resolve env vars: user-supplied values take precedence; defaults fill gaps;
    // required vars with no resolution produce an error.
    let user_env = params_value["env_values"].as_object();
    let mut resolved_env: Vec<(String, String)> = Vec::new();

    for ev in &pkg.environment_variables {
        mcp_params::validate_env_var_name(&ev.name)?;

        let user_val = user_env
            .and_then(|m| m.get(&ev.name))
            .and_then(Value::as_str);

        let resolved = if let Some(v) = user_val {
            mcp_params::validate_env_value(&ev.name, v)?;
            Some(v.to_string())
        } else {
            ev.default.clone()
        };

        match resolved {
            Some(v) => resolved_env.push((ev.name.clone(), v)),
            None if ev.is_required => {
                return Err(ToolError::MissingParam {
                    message: format!(
                        "env var '{}' is required by server '{server_name}' but was not supplied in \
                         `env_values` and has no default",
                        ev.name
                    ),
                    param: ev.name.clone(),
                });
            }
            None => {}
        }
    }

    // Write resolved env vars to ~/.lab/.env if there are any.
    if !resolved_env.is_empty() {
        use crate::config;
        let env_path = config::dotenv_path().ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: "cannot determine ~/.lab/.env path".to_string(),
        })?;
        config::backup_env(&env_path).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!("failed to back up .env: {e}"),
        })?;
        let conflicts = config::write_env_pairs(&env_path, &resolved_env, false).map_err(|e| {
            ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to write env vars to .env: {e}"),
            }
        })?;
        if !conflicts.is_empty() {
            tracing::warn!(
                service = "marketplace",
                action = "mcp.install",
                "env write conflicts (skipped): {}",
                conflicts.join("; ")
            );
        }
    }

    Ok(serde_json::json!({
        "name": gateway_name,
        "url": null,
        "command": hint,
        "args": argv,
        "proxy_resources": false,
        "expose_tools": null,
    }))
}

/// Dispatch `mcp.meta.*` actions that work against the local registry store.
#[cfg(feature = "mcpregistry")]
async fn dispatch_mcp_local(action: &str, params: Value) -> Result<Value, ToolError> {
    use crate::config;
    match action {
        "mcp.meta.get" => {
            let name = mcp_params::require_name(&params)?;
            let requested_version = params["version"].as_str().unwrap_or("latest");
            let store = crate::dispatch::marketplace::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let metadata = store.get_local_metadata(&name, &resolved_version).await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "metadata": metadata,
            }))
        }
        "mcp.meta.set" => {
            let name = mcp_params::require_name(&params)?;
            let requested_version = params["version"].as_str().unwrap_or("latest");
            let metadata =
                params
                    .get("metadata")
                    .cloned()
                    .ok_or_else(|| ToolError::MissingParam {
                        message: "missing required parameter `metadata`".to_string(),
                        param: "metadata".to_string(),
                    })?;
            let metadata = mcp_params::parse_lab_metadata(&metadata)?;
            let updated_by = params
                .get("updated_by")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown");

            let store = crate::dispatch::marketplace::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let metadata_value = serde_json::to_value(metadata)
                .map_err(|e| ToolError::internal_message(format!("serialize lab metadata: {e}")))?;
            store
                .set_local_metadata(&name, &resolved_version, &metadata_value, Some(updated_by))
                .await?;
            let current = store.get_local_metadata(&name, &resolved_version).await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "metadata": current,
            }))
        }
        "mcp.meta.delete" => {
            let name = mcp_params::require_name(&params)?;
            let requested_version = params["version"].as_str().unwrap_or("latest");
            let store = crate::dispatch::marketplace::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let deleted = store
                .delete_local_metadata(&name, &resolved_version)
                .await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "deleted": deleted,
            }))
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: MCP_ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
