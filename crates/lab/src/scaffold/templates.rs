//! Scaffold template loaders.

mod adapters;
mod dispatch;
mod docs;
mod lab_apis;

pub use adapters::{adapter_api_template, adapter_cli_template, adapter_mcp_template};
pub use dispatch::{
    dispatch_catalog_template, dispatch_client_template, dispatch_dispatch_template,
    dispatch_entrypoint_template, dispatch_params_template,
};
pub use docs::coverage_doc_template;
pub use lab_apis::{
    lab_apis_client_template, lab_apis_error_template, lab_apis_service_template,
    lab_apis_types_template,
};

/// Convert a `snake_case` identifier to `PascalCase`.
///
/// Each `_`-separated segment has its first character uppercased; the rest is
/// left unchanged.  An empty segment (e.g. a leading/trailing `_`) produces an
/// empty string for that segment.
pub(crate) fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

/// Replace `{{service}}`, `{{Service}}`, and `{{SERVICE}}` placeholders in a
/// template string with the snake-case service name, its PascalCase form, and
/// its SCREAMING_SNAKE form respectively.
pub(super) fn replace_service(template: &str, service: &str) -> String {
    template
        .replace("{{service}}", service)
        .replace("{{Service}}", &pascal_case(service))
        .replace("{{SERVICE}}", &service.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_case_converts_snake_to_pascal() {
        assert_eq!(pascal_case("my_service"), "MyService");
        assert_eq!(pascal_case("foo"), "Foo");
        assert_eq!(pascal_case("foo_bar_baz"), "FooBarBaz");
    }

    #[test]
    fn pascal_case_empty_segments() {
        // Leading underscore produces an empty first segment.
        assert_eq!(pascal_case("_foo"), "Foo");
        // Empty string produces an empty string.
        assert_eq!(pascal_case(""), "");
    }

    #[test]
    fn replace_service_fills_all_placeholders() {
        let tmpl = "{{service}} {{Service}} {{SERVICE}}";
        assert_eq!(
            replace_service(tmpl, "my_service"),
            "my_service MyService MY_SERVICE"
        );
    }
}
