//! Shared dispatch layer for the `{{service}}` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use client::{client_from_env, not_configured_error, require_client};
pub use dispatch::{dispatch, dispatch_with_client};
