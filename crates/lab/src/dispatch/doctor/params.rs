use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

pub struct ServiceProbeParams<'a> {
    pub service: &'a str,
    pub instance: Option<&'a str>,
}

pub fn parse_service_probe(params: &serde_json::Value) -> Result<ServiceProbeParams<'_>, ToolError> {
    let service = require_str(params, "service")?;
    // Reject any URL in params — resolution must come from env (SSRF defense).
    if service.starts_with("http://") || service.starts_with("https://") {
        return Err(ToolError::InvalidParam {
            message: "service must be a service name, not a URL".to_string(),
            param: "service".to_string(),
        });
    }
    let instance = params
        .get("instance")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty());
    Ok(ServiceProbeParams { service, instance })
}
