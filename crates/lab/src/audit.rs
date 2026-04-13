//! Service onboarding audit.

mod checks;
mod onboarding;
mod types;

#[allow(unused_imports)]
pub use onboarding::{audit_service, audit_services, SharedContext};
pub use types::{AuditReport, CheckResult, ServiceReport};
