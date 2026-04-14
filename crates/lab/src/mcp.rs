//! MCP transport layer — the translation between `lab-apis` clients and
//! the Model Context Protocol. See `crates/lab/src/mcp/CLAUDE.md` for
//! the full rulebook on dispatch, envelopes, and the shared catalog.

pub mod envelope;
pub mod error;
pub mod meta;
pub mod prompts;
pub mod registry;
pub mod resources;
pub mod services;

#[allow(unused_imports)]
pub use envelope::{ToolEnvelope, ToolError};
#[allow(unused_imports)]
pub use registry::ToolRegistry;
