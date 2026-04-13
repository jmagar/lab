//! Templates for onboarding coverage docs.

pub fn coverage_doc_template(service: &str) -> String {
    include_str!("coverage_doc.tpl").replace("{{service}}", service)
}
