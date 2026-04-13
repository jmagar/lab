//! Service onboarding audit.

mod checks;
mod onboarding;
mod types;

#[allow(unused_imports)]
pub use onboarding::{SharedContext, audit_service, audit_services};
pub use types::{AuditReport, CheckResult, ServiceReport};
