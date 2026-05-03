---
name: add-domain
description: Scaffold a new domain module following the spec layout. Creates all required files with correct structure and wires CLI, API, and MCP surfaces.
disable-model-invocation: false
---

## Usage

Called when the user wants to add a new business domain (e.g. `/add-domain sonarr`).

## Steps

1. Ask for: domain name (snake_case), primary subjects (e.g. `series`, `episode`), which actions (`list`, `get`, `create`, `delete`)
2. Create per spec.md layout:
   - `crates/core/src/{domain}.rs` — module entry with `pub mod actions`, `types`, `params`, `handlers`, `client`
   - `crates/core/src/{domain}/actions.rs` — ACTIONS `Vec<ActionSpec>` constant
   - `crates/core/src/{domain}/types.rs` — domain types (split per subject if >1)
   - `crates/core/src/{domain}/params.rs` — param extraction functions
   - `crates/core/src/{domain}/handlers.rs` — `run()` and `run_with_client()` dispatch match
   - `crates/core/src/{domain}/handlers_tests.rs` — sibling test file (NOT inline mod tests)
   - `crates/core/src/{domain}/client.rs` — `{Domain}Client` struct, `Backend` impl, `client_from_env()`
   - `crates/app/src/cli/{domain}.rs` — clap subcommand
   - `crates/app/src/api/services/{domain}.rs` — axum `handle()` fn
3. Wire module into `crates/core/src/lib.rs` and `crates/app/src/lib.rs`
4. Register tool in `crates/app/src/mcp/`

## Invariants to enforce

- No `mod.rs` — use file-per-module convention
- ToolError two-tier only (HttpError → ToolError via From)
- Log fields from `core::log_fields::FIELD_*` constants — no string literals
- Pre-split by subject from day 1 (e.g. `client/series.rs`, not monolithic `client.rs`)
- Action names follow `subject.verb` pattern (e.g. `series.list`, `episode.get`)
- Sibling test files, not inline `mod tests` blocks (unless ≤30 lines trivial)

## Verify after

Invoke the `spec-check` skill to run xtask enforcement gates before declaring the domain complete.
