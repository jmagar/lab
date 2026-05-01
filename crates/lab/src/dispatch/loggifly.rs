//! Shared dispatch layer for the `loggifly` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::client_from_env;
pub use client::not_configured_error;
pub use dispatch::{dispatch, dispatch_with_client};
