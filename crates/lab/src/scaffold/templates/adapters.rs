//! Templates for thin surface adapters.

pub fn adapter_cli_template(service: &str) -> String {
    super::replace_service(include_str!("adapter_cli.tpl"), service)
}

pub fn adapter_mcp_template(service: &str) -> String {
    super::replace_service(include_str!("adapter_mcp.tpl"), service)
}

pub fn adapter_api_template(service: &str) -> String {
    super::replace_service(include_str!("adapter_api.tpl"), service)
}
