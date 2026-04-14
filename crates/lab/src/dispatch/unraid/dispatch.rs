//! Top-level action routing for the `unraid` service.

use lab_apis::unraid::UnraidClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, to_json};
use crate::dispatch::unraid::{
    catalog::ACTIONS,
    client::client_from_instance,
    params::{
        optional_correcting, optional_description, optional_importance, optional_lines,
        optional_type, require_id, require_path, require_title,
    },
};

/// Dispatch using a pre-built client (avoids per-request env reads).
///
/// Called by the API handler which holds a client in `AppState`.
pub async fn dispatch_with_client(
    client: &UnraidClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "system.info" => to_json(client.system_info().await?),
        "system.metrics" => to_json(client.system_metrics().await?),
        "system.array" => to_json(client.system_array().await?),
        "system.online" => {
            let online = client.system_online().await?;
            Ok(serde_json::json!({ "online": online }))
        }
        "docker.list" => to_json(client.docker_list().await?),
        "docker.start" => {
            let id = require_id(&params)?;
            client.docker_start(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "docker.stop" => {
            let id = require_id(&params)?;
            client.docker_stop(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "docker.restart" => {
            let id = require_id(&params)?;
            client.docker_restart(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "disk.list" => to_json(client.disk_list().await?),
        // ---- VM actions ----------------------------------------------------
        "vm.list" => to_json(client.vm_list().await?),
        "vm.start" => {
            let id = require_id(&params)?;
            client.vm_start(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "vm.stop" => {
            let id = require_id(&params)?;
            client.vm_stop(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "vm.pause" => {
            let id = require_id(&params)?;
            client.vm_pause(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "vm.resume" => {
            let id = require_id(&params)?;
            client.vm_resume(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        // ---- Notification actions ------------------------------------------
        "notification.list" => to_json(client.notification_list().await?),
        "notification.create" => {
            let title = require_title(&params)?;
            let description = optional_description(&params)?;
            let importance = optional_importance(&params)?;
            let notification_type = optional_type(&params)?;
            client
                .notification_create(
                    &title,
                    description.as_deref(),
                    importance.as_deref(),
                    notification_type.as_deref(),
                )
                .await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "notification.archive" => {
            let id = require_id(&params)?;
            client.notification_archive(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        // ---- Parity actions ------------------------------------------------
        "parity.history" => to_json(client.parity_history().await?),
        "parity.check-start" => {
            let correcting = optional_correcting(&params)?;
            client.parity_check_start(correcting).await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "parity.check-pause" => {
            client.parity_check_pause().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "parity.check-cancel" => {
            client.parity_check_cancel().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        // ---- Share / plugin / network / UPS --------------------------------
        "share.list" => to_json(client.share_list().await?),
        "plugin.list" => to_json(client.plugin_list().await?),
        "network.list" => to_json(client.network_list().await?),
        "ups.devices" => to_json(client.ups_devices().await?),
        "ups.config" => to_json(client.ups_config().await?),
        // ---- Log file ------------------------------------------------------
        "log.read" => {
            let path = require_path(&params)?;
            let lines = optional_lines(&params)?;
            let content = client.log_read(&path, lines).await?;
            Ok(serde_json::json!({ "content": content }))
        }
        // ---- Flash operations ----------------------------------------------
        "flash.status" => to_json(client.flash_status().await?),
        "flash.backup" => {
            client.flash_backup().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `unraid` service.
///
/// Handles the built-in `help` and `schema` actions before delegating to
/// `dispatch_with_client`.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("unraid", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ => {}
    }

    if !ACTIONS.iter().any(|spec| spec.name == action) {
        return Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unraid`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        });
    }

    let instance = optional_str(&params, "instance")?.map(str::to_owned);
    let mut params = params;
    if let Value::Object(ref mut map) = params {
        map.remove("instance");
    }

    match instance {
        Some(label) => {
            let client = client_from_instance(Some(&label))?;
            dispatch_with_client(&client, action, params).await
        }
        None => match client_from_instance(None) {
            Ok(client) => dispatch_with_client(&client, action, params).await,
            Err(ToolError::UnknownInstance { valid, .. }) if valid.is_empty() => {
                Err(crate::dispatch::unraid::not_configured_error())
            }
            Err(err) => Err(err),
        },
    }
}
