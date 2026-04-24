//! Shared dispatch layer for the unified marketplace service.
//!
//! Covers three item types:
//! - Plugins (Claude Code marketplaces, cherry-pick from installed plugins)
//! - MCP Servers (from MCP Registry CDN — absorbed from dispatch/mcpregistry)
//! - ACP Agents (from cdn.agentclientprotocol.com registry JSON)

mod acp_catalog;
mod acp_client;
mod acp_dispatch;
mod backend;
mod backends;
mod catalog;
mod client;
mod dispatch;
mod mcp_catalog;
mod mcp_client;
mod mcp_dispatch;
mod mcp_params;
mod package;
mod params;
mod runtime;
pub(crate) mod service;
#[cfg(feature = "mcpregistry")]
pub mod store;
#[cfg(feature = "mcpregistry")]
pub mod sync;

pub use catalog::ACTIONS;
pub use dispatch::dispatch;
#[cfg(feature = "mcpregistry")]
pub use mcp_params::{resolve_search_for_rest, validate_registry_url};
pub const LAB_REGISTRY_META_NAMESPACE: &str = "tv.tootie.lab/registry";
