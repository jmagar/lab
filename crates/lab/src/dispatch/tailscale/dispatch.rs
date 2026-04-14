use serde_json::Value;

use lab_apis::tailscale::TailscaleClient;

use super::catalog::ACTIONS;
use super::params;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Dispatch against a pre-built `TailscaleClient` (avoids per-request env reads).
///
/// Called by the API handler with the client from `AppState`.
#[allow(clippy::too_many_lines)]
pub async fn dispatch_with_client(
    client: &TailscaleClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Devices ──────────────────────────────────────────────────────────
        "device.list" => to_json(client.devices_list().await?),
        "device.get" => {
            let device_id = params::device_id_from_params(&params_value)?;
            to_json(client.device_get(&device_id).await?)
        }
        "device.delete" => {
            let device_id = params::device_id_from_params(&params_value)?;
            client.device_delete(&device_id).await?;
            Ok(Value::Null)
        }
        "device.authorize" => {
            let device_id = params::device_id_from_params(&params_value)?;
            let authorized = params::authorized_from_params(&params_value)?;
            client.device_authorize(&device_id, authorized).await?;
            Ok(Value::Null)
        }
        // ── Auth Keys ────────────────────────────────────────────────────────
        "key.list" => to_json(client.keys_list().await?),
        "key.get" => {
            let key_id = params::key_id_from_params(&params_value)?;
            to_json(client.key_get(&key_id).await?)
        }
        "key.delete" => {
            let key_id = params::key_id_from_params(&params_value)?;
            client.key_delete(&key_id).await?;
            Ok(Value::Null)
        }
        // ── DNS ──────────────────────────────────────────────────────────────
        "dns.nameservers" => to_json(client.dns_nameservers().await?),
        "dns.search_paths" => to_json(client.dns_search_paths().await?),
        "dns.split-get" => to_json(client.dns_split_get().await?),
        "dns.split-set" => {
            let config = params::required_object(&params_value, "config")?;
            to_json(client.dns_split_set(config).await?)
        }
        // ── ACL / Policy ─────────────────────────────────────────────────────
        "acl.get" => to_json(client.acl_get().await?),
        "acl.validate" => {
            let policy = params::required_object(&params_value, "policy")?;
            to_json(client.acl_validate(policy).await?)
        }
        "acl.set" => {
            let policy = params::required_object(&params_value, "policy")?;
            // Safety: validate the policy first — if validation returns errors, abort.
            let validation = client.acl_validate(policy).await?;
            if let Some(errors) = validation.get("message").filter(|v| !v.is_null()) {
                // API returned a non-null message field — treat as validation failure
                return Err(ToolError::Sdk {
                    sdk_kind: "validation_failed".to_string(),
                    message: format!("ACL validation failed: {errors}"),
                });
            }
            // Also check for an "errors" array (Tailscale API v2 shape)
            if let Some(Value::Array(errs)) = validation.get("errors")
                && !errs.is_empty()
            {
                return Err(ToolError::Sdk {
                    sdk_kind: "validation_failed".to_string(),
                    message: format!("ACL validation errors: {validation}"),
                });
            }
            to_json(client.acl_set(policy).await?)
        }
        // ── Device extended ops ───────────────────────────────────────────────
        "device.routes-get" => {
            let device_id = params::device_id_from_params(&params_value)?;
            to_json(client.device_routes_get(&device_id).await?)
        }
        "device.routes-set" => {
            let device_id = params::device_id_from_params(&params_value)?;
            let routes = params::required_string_array(&params_value, "routes")?;
            to_json(client.device_routes_set(&device_id, &routes).await?)
        }
        "device.tag" => {
            let device_id = params::device_id_from_params(&params_value)?;
            let tags = params::required_string_array(&params_value, "tags")?;
            client.device_tag(&device_id, &tags).await?;
            Ok(Value::Null)
        }
        "device.expire" => {
            let device_id = params::device_id_from_params(&params_value)?;
            client.device_expire(&device_id).await?;
            Ok(Value::Null)
        }
        // ── Users ─────────────────────────────────────────────────────────────
        "user.list" => to_json(client.user_list().await?),
        // ── Tailnet Settings ──────────────────────────────────────────────────
        "tailnet.settings-get" => to_json(client.tailnet_settings_get().await?),
        "tailnet.settings-patch" => {
            let settings = params::required_object(&params_value, "settings")?;
            to_json(client.tailnet_settings_patch(settings).await?)
        }
        // ── Key extended ──────────────────────────────────────────────────────
        "key.create" => {
            let capabilities = params::required_object(&params_value, "capabilities")?;
            let expiry_seconds = params_value.get("expiry_seconds").and_then(Value::as_u64);
            let description = params_value
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let req = lab_apis::tailscale::types::CreateKeyRequest {
                capabilities: capabilities.clone(),
                expiry_seconds,
                description,
            };
            // Note: response includes key string — caller is responsible for secure handling
            to_json(client.key_create(&req).await?)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Top-level dispatch: handles built-ins then routes to `dispatch_with_client`.
///
/// Used by MCP and CLI surfaces.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("tailscale", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}'"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    let client = super::client::require_client()?;
    dispatch_with_client(&client, action, params_value).await
}
