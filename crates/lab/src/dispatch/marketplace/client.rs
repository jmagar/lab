//! No external HTTP client — `marketplace` is a local-only synthetic service.
//!
//! This file satisfies the required dispatch service layout contract
//! (every migrated service must have `client.rs`). All work is local
//! filesystem I/O plus optional `tokio::process::Command` shell-out to
//! `claude plugin install/uninstall`.
