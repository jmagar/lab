//! Per-service dispatch modules.
//!
//! Each enabled service declares one submodule here, feature-gated with
//! `#[cfg(feature = "<service>")]`. The submodule exposes a `dispatch`
//! function that takes an action + params object and returns a
//! [`crate::mcp::envelope::ToolEnvelope`] or [`crate::mcp::envelope::ToolError`].
//!
//! No services are wired in this skeleton — they are added in later
//! service-specific plans.
