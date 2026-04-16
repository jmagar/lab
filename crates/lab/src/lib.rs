#![allow(clippy::multiple_crate_versions)]
#![allow(unreachable_pub)]

pub mod api;
pub mod audit;
pub mod catalog;
pub mod cli;
pub mod config;
pub mod device;
pub mod dispatch;
pub mod mcp;
pub mod oauth;
pub mod output;
pub mod registry;
pub mod scaffold;
#[cfg(test)]
pub mod test_support;
pub mod tui;
