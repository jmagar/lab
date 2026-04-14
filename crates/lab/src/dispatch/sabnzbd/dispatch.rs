use lab_apis::sabnzbd::SabnzbdClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{opt_u32, opt_str, require_str, require_u64, to_json};

/// Redact sensitive fields from the `SABnzbd` config response.
///
/// Replaces the following with `"[redacted]"`:
/// - top-level `api_key`
/// - `servers[*].password`
/// - `indexers[*].apikey`
pub fn sanitize_config(mut value: Value) -> Value {
    if let Some(obj) = value.as_object_mut() {
        // Redact top-level api_key
        if obj.contains_key("api_key") {
            obj.insert("api_key".to_string(), Value::String("[redacted]".to_string()));
        }
        // Redact servers[*].password
        if let Some(Value::Array(servers)) = obj.get_mut("servers") {
            for server in servers.iter_mut() {
                if let Some(s) = server.as_object_mut()
                    && s.contains_key("password") {
                        s.insert("password".to_string(), Value::String("[redacted]".to_string()));
                    }
            }
        }
        // Redact indexers[*].apikey
        if let Some(Value::Array(indexers)) = obj.get_mut("indexers") {
            for indexer in indexers.iter_mut() {
                if let Some(idx) = indexer.as_object_mut()
                    && idx.contains_key("apikey") {
                        idx.insert("apikey".to_string(), Value::String("[redacted]".to_string()));
                    }
            }
        }
    }
    value
}

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &SabnzbdClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "server.version" => {
            let version = client.version().await?;
            Ok(serde_json::json!({ "version": version }))
        }
        "queue.list" => to_json(client.queue().await?),
        "queue.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = client.queue_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }
        "history.list" => {
            let limit = opt_u32(&params, "limit")?;
            to_json(client.history(limit).await?)
        }
        "history.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = client.history_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }
        "history.purge" => {
            let ok = client.history_purge().await?;
            Ok(serde_json::json!({ "purged": ok }))
        }
        "server.stats" => to_json(client.server_stats().await?),
        "server.warnings" => to_json(client.warnings().await?),
        "queue.pause" => {
            let ok = client.pause().await?;
            Ok(serde_json::json!({ "paused": ok }))
        }
        "queue.resume" => {
            let ok = client.resume().await?;
            Ok(serde_json::json!({ "resumed": ok }))
        }
        "queue.speed.limit" => {
            let kbps = require_u64(&params, "kbps")?;
            let ok = client.set_speed_limit(kbps).await?;
            Ok(serde_json::json!({ "speed_limit_set": ok }))
        }
        "queue.addurl" => {
            let url = require_str(&params, "url")?;
            let cat = opt_str(&params, "cat");
            let priority = opt_str(&params, "priority");
            let resp = client.queue_addurl(url, cat, priority).await?;
            Ok(resp)
        }
        "history.retry" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = client.history_retry(nzo_id).await?;
            Ok(serde_json::json!({ "retried": ok }))
        }
        "history.retry-all" => {
            let ok = client.history_retry_all().await?;
            Ok(serde_json::json!({ "retried_all": ok }))
        }
        "server.fullstatus" => {
            let resp = client.server_fullstatus().await?;
            Ok(resp)
        }
        "category.list" => {
            let resp = client.category_list().await?;
            Ok(resp)
        }
        "queue.set-complete-action" => {
            let action_value = require_str(&params, "action")?;
            let ok = client.queue_set_complete_action(action_value).await?;
            Ok(serde_json::json!({ "complete_action_set": ok }))
        }
        "pp.pause" => {
            let ok = client.pp_pause().await?;
            Ok(serde_json::json!({ "pp_paused": ok }))
        }
        "pp.resume" => {
            let ok = client.pp_resume().await?;
            Ok(serde_json::json!({ "pp_resumed": ok }))
        }
        "rss.fetch-now" => {
            let resp = client.rss_fetch_now().await?;
            Ok(resp)
        }
        "config.get" => {
            let raw = client.config_get().await?;
            Ok(sanitize_config(raw))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `sabnzbd`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("sabnzbd", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action `{action}` for service `sabnzbd`"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&require_client()?, action, params).await
}

#[cfg(test)]
mod tests {
    use super::sanitize_config;
    use serde_json::json;

    #[test]
    fn sanitize_config_redacts_api_key() {
        let raw = json!({ "api_key": "super-secret", "misc": {} });
        let out = sanitize_config(raw);
        assert_eq!(out["api_key"], "[redacted]");
        assert!(out["misc"].is_object());
    }

    #[test]
    fn sanitize_config_redacts_server_passwords() {
        let raw = json!({
            "servers": [
                { "host": "news.example.com", "password": "hunter2" },
                { "host": "news2.example.com", "password": "pass123" }
            ]
        });
        let out = sanitize_config(raw);
        assert_eq!(out["servers"][0]["password"], "[redacted]");
        assert_eq!(out["servers"][1]["password"], "[redacted]");
        assert_eq!(out["servers"][0]["host"], "news.example.com");
    }

    #[test]
    fn sanitize_config_redacts_indexer_apikeys() {
        let raw = json!({
            "indexers": [
                { "name": "NZBGeek", "apikey": "geek-key" },
                { "name": "NZBPlanet", "apikey": "planet-key" }
            ]
        });
        let out = sanitize_config(raw);
        assert_eq!(out["indexers"][0]["apikey"], "[redacted]");
        assert_eq!(out["indexers"][1]["apikey"], "[redacted]");
        assert_eq!(out["indexers"][0]["name"], "NZBGeek");
    }

    #[test]
    fn sanitize_config_handles_missing_sensitive_fields() {
        let raw = json!({ "config": { "misc": { "host": "0.0.0.0" } } });
        let out = sanitize_config(raw);
        assert_eq!(out["config"]["misc"]["host"], "0.0.0.0");
    }
}
