//! Dispatch layer for the `deploy` service.
//!
//! All orchestration for the deploy flow lives here. The `mcp/services/deploy.rs`
//! and `cli/deploy.rs` surfaces are thin adapters that go through this module.
//!
//! Scaffold modules (`build`, `lock`) are exported for the follow-up runner
//! wiring; `#[allow(dead_code)]` keeps the V1 scaffold from tripping the
//! workspace's `-D warnings` lint while the integration lands.

#![allow(dead_code)]

pub mod authz;
pub mod build;
pub mod catalog;
pub mod dispatch;
pub mod lock;
pub mod params;
pub mod runner;
pub mod ssh_session;

#[allow(unused_imports)]
pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_mcp, dispatch_with_runner};
