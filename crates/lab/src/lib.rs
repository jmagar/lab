#![allow(clippy::multiple_crate_versions)]

pub mod acp;
#[allow(unreachable_pub)]
pub mod api;
#[allow(unreachable_pub)]
pub mod audit;
pub mod catalog;
#[allow(unreachable_pub)]
pub mod cli;
pub mod config;
#[allow(unreachable_pub)]
pub mod dispatch;
#[allow(unreachable_pub)]
pub mod mcp;
#[allow(unreachable_pub)]
pub mod node;
#[allow(unreachable_pub)]
pub mod oauth;
pub mod output;
#[allow(unreachable_pub)]
pub mod process;
#[allow(unreachable_pub)]
pub mod registry;
#[allow(unreachable_pub)]
pub mod scaffold;
#[cfg(test)]
pub mod test_support;
#[allow(unreachable_pub)]
pub mod tui;
