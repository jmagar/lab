//! Templates for `crates/lab-apis/src/<service>`.

pub fn lab_apis_service_template(service: &str) -> String {
    super::replace_service(include_str!("lab_apis_service.tpl"), service)
}

pub fn lab_apis_client_template(service: &str) -> String {
    super::replace_service(include_str!("lab_apis_client.tpl"), service)
}

pub fn lab_apis_types_template(service: &str) -> String {
    super::replace_service(include_str!("lab_apis_types.tpl"), service)
}

pub fn lab_apis_error_template(service: &str) -> String {
    super::replace_service(include_str!("lab_apis_error.tpl"), service)
}
