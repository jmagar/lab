use serde_json::Value;

use lab_apis::openacp::types::{
    AdoptSessionRequest, BypassRequest, ConfigPatchRequest, CreateSessionRequest, NotifyRequest,
    PermissionResolveRequest, PromptRequest, TopicsCleanupRequest, TunnelCreateRequest,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, optional_u32, require_str};

pub fn optional_instance(params: &Value) -> Result<Option<&str>, ToolError> {
    optional_str(params, "instance")
}

pub fn session_id(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "session_id")
}

pub fn create_session(params: &Value) -> Result<CreateSessionRequest, ToolError> {
    Ok(CreateSessionRequest {
        agent: optional_str(params, "agent")?.map(ToOwned::to_owned),
        workspace: optional_str(params, "workspace")?.map(ToOwned::to_owned),
    })
}

pub fn prompt(params: &Value) -> Result<PromptRequest, ToolError> {
    Ok(PromptRequest {
        prompt: require_str(params, "prompt")?.to_string(),
    })
}

pub fn bypass(params: &Value) -> Result<BypassRequest, ToolError> {
    let enabled = params
        .get("enabled")
        .and_then(Value::as_bool)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `enabled`".to_string(),
            param: "enabled".to_string(),
        })?;
    Ok(BypassRequest { enabled })
}

pub fn permission(params: &Value) -> Result<PermissionResolveRequest, ToolError> {
    Ok(PermissionResolveRequest {
        permission_id: require_str(params, "permission_id")?.to_string(),
        option_id: require_str(params, "option_id")?.to_string(),
    })
}

pub fn adopt(params: &Value) -> Result<AdoptSessionRequest, ToolError> {
    Ok(AdoptSessionRequest {
        agent: require_str(params, "agent")?.to_string(),
        agent_session_id: require_str(params, "agent_session_id")?.to_string(),
        cwd: optional_str(params, "cwd")?.map(ToOwned::to_owned),
        channel: optional_str(params, "channel")?.map(ToOwned::to_owned),
    })
}

pub fn config_patch(params: &Value) -> Result<ConfigPatchRequest, ToolError> {
    let value = params
        .get("value")
        .cloned()
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `value`".to_string(),
            param: "value".to_string(),
        })?;
    Ok(ConfigPatchRequest {
        path: require_str(params, "path")?.to_string(),
        value,
    })
}

pub fn topics_cleanup(params: &Value) -> Result<TopicsCleanupRequest, ToolError> {
    let statuses = match params.get("statuses") {
        None => None,
        Some(Value::Array(values)) => Some(
            values
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| ToolError::InvalidParam {
                            message: "parameter `statuses` must be an array of strings".to_string(),
                            param: "statuses".to_string(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Some(_) => {
            return Err(ToolError::InvalidParam {
                message: "parameter `statuses` must be an array of strings".to_string(),
                param: "statuses".to_string(),
            });
        }
    };
    Ok(TopicsCleanupRequest { statuses })
}

pub fn optional_force(params: &Value) -> Result<bool, ToolError> {
    params.get("force").map_or(Ok(false), |value| {
        value.as_bool().ok_or_else(|| ToolError::InvalidParam {
            message: "parameter `force` must be a boolean".to_string(),
            param: "force".to_string(),
        })
    })
}

pub fn tunnel_create(params: &Value) -> Result<TunnelCreateRequest, ToolError> {
    let port = optional_u32(params, "port")?.ok_or_else(|| ToolError::MissingParam {
        message: "missing required parameter `port`".to_string(),
        param: "port".to_string(),
    })?;
    let port = u16::try_from(port).map_err(|_| ToolError::InvalidParam {
        message: "parameter `port` must be between 0 and 65535".to_string(),
        param: "port".to_string(),
    })?;
    Ok(TunnelCreateRequest {
        port,
        label: optional_str(params, "label")?.map(ToOwned::to_owned),
        session_id: optional_str(params, "session_id")?.map(ToOwned::to_owned),
    })
}

pub fn port(params: &Value) -> Result<u16, ToolError> {
    let port = optional_u32(params, "port")?.ok_or_else(|| ToolError::MissingParam {
        message: "missing required parameter `port`".to_string(),
        param: "port".to_string(),
    })?;
    u16::try_from(port).map_err(|_| ToolError::InvalidParam {
        message: "parameter `port` must be between 0 and 65535".to_string(),
        param: "port".to_string(),
    })
}

pub fn notify(params: &Value) -> Result<NotifyRequest, ToolError> {
    Ok(NotifyRequest {
        message: require_str(params, "message")?.to_string(),
    })
}
