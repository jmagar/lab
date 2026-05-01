# Technology

This document captures the locked stack and tooling choices for `lab`.

## Language and Packaging

- Rust 2024 edition
- single workspace version
- dual MIT / Apache-2.0 license
- targets: Linux x86_64, Linux aarch64
- workspace resolver 3

## Core Runtime

| Concern | Choice |
|---|---|
| async runtime | `tokio` |
| concurrency helpers | `futures` |
| HTTP client | `reqwest` with rustls |
| URL handling | `url::Url` |
| serialization | `serde`, `serde_json` |
| library errors | `thiserror` |
| binary errors | `anyhow` |
| time | `jiff` |
| logging | `tracing`, `tracing-subscriber` |

`url::Url` is the canonical URL type. Service code should not pass base URLs around as unvalidated strings once a client is constructed.

## Product Surfaces

| Concern | Choice |
|---|---|
| CLI | `clap` |
| MCP server | `rmcp` |
| TUI | `ratatui`, `crossterm` |
| tabular output | `tabled` |
| color | `owo-colors` |
| TTY detection | `is-terminal` |
| progress bars | `indicatif` |

## Config and Bootstrap

| Concern | Choice |
|---|---|
| `.env` loading | `dotenvy` |
| TOML config | `toml` |
| SSH transport for extract | `russh`, `russh-keys` |
| XML parsing for extract | `quick-xml` |

## Testing and Quality

| Concern | Choice |
|---|---|
| unit HTTP mocking | `wiremock` |
| snapshots | `insta` |
| test runner | `cargo-nextest` |
| linting | `clippy`, `rustfmt`, `cargo-deny` |
| task runner | `just` |
| CI | GitHub Actions |

## Workspace Rules

- dependency versions live at the workspace root
- lints live at the workspace root
- feature flags are mirrored from `lab-apis` into `lab`
- release profile is optimized and stripped
- dev profile keeps faster local iteration
- release-debug profile exists for profiling and diagnostics
- `unsafe` is forbidden at the workspace lint layer

## Feature Gating

`lab-apis` owns the actual service feature flags. `lab` re-exports them one-for-one so a single feature selection controls both crates.

The practical rules are:

- `default` enables all service integrations
- `all` enables every service integration
- internal shared features like `servarr` stay implementation details
- `extract` is always compiled

## Build and Verify

Primary commands:

```bash
just check
just test
just lint
just deny
just build
```

Scoped commands:

```bash
cargo test -p lab-apis
cargo test --manifest-path crates/lab/Cargo.toml
```

Documentation verification target:

```bash
cargo doc --no-deps --all-features
```

## CI Model

CI is intended to cover:

- `cargo check --workspace --all-features`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-features -- -D warnings`
- `cargo deny check`
- `cargo nextest run --workspace --all-features`
- docs without rustdoc warnings where enabled

Rustdoc should also stay warning-free when enabled.

More operational detail lives in [OPERATIONS.md](./OPERATIONS.md).

## Release Tooling

- `cargo-release` for versioning and tagging
- GitHub-generated release notes
- GitHub Actions for release builds
- GitHub Releases for artifacts
- no automatic update checks at startup

## Non-Goals

- no telemetry
- no background analytics
- no analytics or telemetry phone-home to third-party services; first-party node-to-controller fleet reporting is intentional runtime behavior

That is a product rule, not just a tooling preference.
