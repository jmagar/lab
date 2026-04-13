//! Templates for `crates/lab/src/dispatch/<service>`.

pub fn dispatch_entrypoint_template(service: &str) -> String {
    super::replace_service(include_str!("dispatch_entrypoint.tpl"), service)
}

pub fn dispatch_catalog_template(service: &str) -> String {
    super::replace_service(include_str!("dispatch_catalog.tpl"), service)
}

pub fn dispatch_client_template(service: &str) -> String {
    super::replace_service(include_str!("dispatch_client.tpl"), service)
}

pub fn dispatch_dispatch_template(service: &str) -> String {
    super::replace_service(include_str!("dispatch_dispatch.tpl"), service)
}

pub fn dispatch_params_template(service: &str) -> String {
    super::replace_service(include_str!("dispatch_params.tpl"), service)
}
