//! Bootstrap utility: extract API keys and URLs from existing service config files.
//!
//! `extract` is a synthetic service — it has the same module shape as a real
//! API client (`client.rs`, `types.rs`, `error.rs`) but it reads files instead
//! of speaking HTTP. Local paths and remote `host:/path` URIs both work; SSH
//! is built in via `russh`.
//!
//! The pure capability lives here. The lab-specific glue (writing results to
//! `~/.lab/.env`, prompting on destructive apply, formatting the table) lives
//! in `crates/lab/src/cli/extract.rs` and `crates/lab/src/mcp/services/extract.rs`.
//!
//! ```no_run
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! use lab_apis::extract::ExtractClient;
//!
//! let client = ExtractClient::new();
//! let report = client.scan("squirts:/mnt/appdata".parse()?).await?;
//! for cred in report.creds {
//!     println!("{}: {}", cred.service, cred.url.as_deref().unwrap_or("?"));
//! }
//! # Ok(()) }
//! ```

/// Pluggable per-app config parsers.
pub mod parsers;

/// File-reading abstraction over local fs and ssh.
pub mod transport;

/// Public types: `Uri`, `ExtractedCreds`, `ServiceCreds`, `ExtractReport`.
pub mod types;

/// `ExtractError` (thiserror).
pub mod error;

/// `ExtractClient` — orchestrates URI parsing, transport selection, and parser dispatch.
pub mod client;

pub use client::ExtractClient;
pub use error::ExtractError;
pub use types::{ExtractReport, ExtractWarning, ServiceCreds, Uri};

use crate::core::plugin::{Category, PluginMeta};

/// Compile-time metadata for the extract module.
pub const META: PluginMeta = PluginMeta {
    name: "extract",
    display_name: "Extract",
    description: "Pull API keys and URLs from existing service config files",
    category: Category::Bootstrap,
    docs_url: "",
    required_env: &[],
    optional_env: &[],
    default_port: None,
};
