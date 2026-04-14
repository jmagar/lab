//! Re-export upstream types from the shared dispatch layer.
//!
//! The canonical implementation lives in `crate::dispatch::upstream`.
//! This re-export exists so that existing `crate::mcp::upstream::*`
//! references continue to resolve without a full codebase rename.

#[allow(unused_imports)]
pub use crate::dispatch::upstream::{pool, types};
