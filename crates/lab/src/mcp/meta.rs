//! The `lab.help` global MCP meta-tool. Returns the full catalog in
//! envelope form so agents can discover every enabled service and
//! action in one call.

use crate::{
    catalog::{Catalog, build_catalog},
    mcp::{envelope::ToolEnvelope, registry::ToolRegistry},
};

/// Dispatch the `lab.help` meta-tool.
pub fn help(registry: &ToolRegistry) -> ToolEnvelope<Catalog> {
    ToolEnvelope::new(build_catalog(registry))
}
