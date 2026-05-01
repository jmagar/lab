//! Shared dispatch layer for the `beads` service.

pub mod catalog;
pub mod client;
pub mod dispatch;
pub mod params;

pub use catalog::ACTIONS;
pub use client::{client_from_env, not_configured_error};
pub use dispatch::{dispatch, dispatch_with_client};
