#![allow(clippy::multiple_crate_versions)]

#[allow(unreachable_pub)]
pub mod api;
#[allow(unreachable_pub)]
pub mod audit;
pub mod catalog;
#[allow(unreachable_pub)]
pub mod cli;
pub mod config;
#[allow(unreachable_pub)]
pub mod device;
#[allow(unreachable_pub)]
pub mod dispatch;
#[allow(unreachable_pub)]
pub mod mcp;
#[allow(unreachable_pub)]
pub mod oauth;
pub mod output;
#[allow(unreachable_pub)]
pub mod registry;
#[allow(unreachable_pub)]
pub mod scaffold;
#[cfg(test)]
pub mod test_support;
#[allow(unreachable_pub)]
pub mod tui;
