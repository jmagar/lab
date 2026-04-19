//! Dispatch layer for the `deploy` service.
//!
//! All orchestration for the deploy flow lives here. The `mcp/services/deploy.rs`
//! and `cli/deploy.rs` surfaces are thin adapters that go through this module.

pub mod authz;
pub mod build;
pub mod catalog;
pub mod client;
pub mod dispatch;
pub mod lock;
pub mod params;
pub mod runner;
pub mod ssh_session;

#[allow(unused_imports)]
pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_mcp, dispatch_with_runner};
