# Lab — Development Instructions

## What is this?

`lab` is a pluggable homelab CLI + MCP server SDK in Rust. One binary, **21 services** (20 feature-gated + always-on `extract`), runtime MCP tool selection via a single tool per service with an `action` + `params` dispatch shape (~20 MCP tools max, not hundreds).

Read `docs/DESIGN.md` for the full architecture and decisions. **`docs/DESIGN.md` is the source of truth** — if this file disagrees, DESIGN wins and this file is stale.

**Nested guides.** Subdirectories carry their own `CLAUDE.md` with rules that don't belong at the root. Read the nearest one when working in:
- `crates/lab-apis/src/core/` — trait contracts, error taxonomy, HttpClient invariants
- `crates/lab-apis/src/servarr/` — shared *arr primitives
- `crates/lab-apis/src/extract/` — synthetic-service rules, `.env` merge algorithm
- `crates/lab/src/mcp/` — dispatch, envelopes, elicitation, catalog
- `crates/lab/src/cli/` — thin-shim pattern, destructive flags, batch commands
- `crates/lab/src/tui/` — plugin manager UX, `.mcp.json` patching

## Repository Structure

Two crates. Pure API clients live in `lab-apis`. Everything else (CLI, MCP, TUI, binary) lives in `lab`.

```
lab/
├── crates/
│   ├── lab-apis/                     # PURE Rust SDK — reusable in any binary
│   │   ├── Cargo.toml                # deps: reqwest, serde, thiserror, tokio
│   │   └── src/
│   │       ├── lib.rs                # re-exports, feature gates
│   │       ├── core/                 # HttpClient, Auth, errors, traits
│   │       ├── servarr/              # shared *arr primitives
│   │       ├── radarr/               # { client.rs, types.rs, error.rs }
│   │       ├── sonarr/
│   │       ├── prowlarr/
│   │       ├── plex/
│   │       ├── tautulli/
│   │       ├── sabnzbd/
│   │       ├── qbittorrent/
│   │       ├── tailscale/
│   │       ├── linkding/
│   │       ├── memos/
│   │       ├── bytestash/
│   │       ├── paperless/
│   │       ├── arcane/                # Docker management UI
│   │       ├── unraid/                # Unraid GraphQL API
│   │       ├── unifi/                 # UniFi Network Application local API
│   │       ├── overseerr/              # Media request manager
│   │       ├── gotify/                # Push notifications
│   │       ├── openai/                # OpenAI API (+ OpenAI-compatible)
│   │       ├── qdrant/                # Vector database
│   │       ├── tei/                   # HF Text Embeddings Inference
│   │       ├── apprise/               # Universal notification dispatcher
│   │       └── extract/                # ALWAYS-ON synthetic service: scan local/SSH hosts for service creds
│   │
│   └── lab/                          # BINARY: cli + mcp + tui + main
│       ├── Cargo.toml                # deps: lab-apis, clap, rmcp, ratatui, anyhow, tabled
│       └── src/
│           ├── main.rs
│           ├── cli/                  # clap subcommands per service (thin shims)
│           ├── mcp/
│           │   ├── registry.rs       # runtime tool registration
│           │   ├── resources.rs      # action catalog as MCP resources
│           │   ├── error.rs          # structured JSON errors
│           │   └── services/         # one dispatch module per service
│           ├── tui/                  # ratatui plugin manager
│           ├── config.rs             # ~/.lab/.env + config.toml loading
│           └── output.rs             # table/json formatting
├── Cargo.toml                        # workspace
├── Justfile
├── deny.toml
├── docs/DESIGN.md
└── CLAUDE.md
```

## Key Patterns

### Per-Service Module Structure (in `lab-apis`)

Every service is a module under `crates/lab-apis/src/`:

```
foo.rs              # module declaration: pub mod client; pub mod types; pub mod error; pub const META: ...
foo/
├── client.rs       # FooClient with async methods — ALL business logic
├── types.rs        # Request/response types (serde)
└── error.rs        # Service-specific errors (thiserror)
```

Modern Rust module style: **no `mod.rs` files anywhere**. A module `foo` is declared in `foo.rs` (sibling to the `foo/` directory), not in `foo/mod.rs`.

Note: `commands.rs` and `tools.rs` do **not** live here. CLI subcommands and MCP dispatch live in the `lab` crate, never in `lab-apis`.

### The Golden Rule

Business logic lives in `lab-apis/src/<service>/client.rs`. CLI (`lab/src/cli/<service>.rs`) and MCP (`lab/src/mcp/services/<service>.rs`) are **thin shims** that call client methods and format output. If you're writing logic in a CLI command or MCP dispatch, you're doing it wrong — move it to the client.

The two-crate split enforces this structurally: `lab-apis` doesn't depend on `clap` or `rmcp`, so you literally cannot reach for them while writing business logic.

### One Tool Per Service (MCP) — action + subaction dispatch

Each service exposes exactly **one** MCP tool, named after the service. Operations dispatch via a flat dotted `action` string + free-form `params` object. This keeps total MCP tool count at ~20, not hundreds.

```jsonc
radarr({ "action": "movie.search", "params": { "query": "The Matrix" } })
radarr({ "action": "queue.list" })
radarr({ "action": "help" })                        // built-in discovery
radarr({ "action": "schema", "params": { "action": "movie.add" } })  // per-action schema
```

- **Action naming:** `<resource>.<verb>`, lowercase, dot-separated.
- **Built-in actions:** every tool accepts `help` and `schema` without declaring them.
- **Discovery:** `lab://<service>/actions` MCP resource + global `lab.help` meta-tool + `lab://catalog` resource.
- **Shared catalog.** `build_catalog()` is a single function feeding three surfaces: the `lab.help` MCP tool, the `lab://catalog` MCP resource, and the `lab help` CLI subcommand. Never duplicate catalog logic — extend the builder.
- **Multi-instance services.** When `S_<LABEL>_URL` env vars exist, callers pass `params.instance: "<label>"`. Unknown labels return a structured `unknown_instance` envelope listing valid labels.

### Destructive actions

`ActionSpec.destructive: bool` is the **single source of truth** for dangerous operations. It drives:

- **MCP:** elicitation — the dispatcher prompts the client to confirm before executing.
- **CLI:** requires `-y` / `--yes` to run non-interactively. `--no-confirm` and `--dry-run` are also honored.

Mark actions `destructive: true` whenever they delete, overwrite, or push state that can't be trivially reversed (`extract.apply`, `radarr.movie.delete`, `sabnzbd.queue.purge`, etc.).

### Structured error envelopes

Every MCP tool failure returns a JSON envelope with a stable `kind` tag so agents can react programmatically:

```jsonc
{ "kind": "unknown_action", "message": "...", "valid": ["movie.search", ...], "hint": "movie.serch" }
{ "kind": "missing_param",  "message": "...", "param": "query" }
{ "kind": "unknown_instance", "message": "...", "valid": ["default", "node2"] }
{ "kind": "rate_limited", "message": "...", "retry_after_ms": 5000 }
```

See `docs/DESIGN.md` for the full MCP spec and envelope taxonomy.

### Adding a New Service

1. `mkdir crates/lab-apis/src/foo/`
2. Define types in `types.rs` from API spec/docs
3. Implement `FooClient` methods in `client.rs`
4. Implement `ServiceClient` trait for health checks
5. Add `#[cfg(feature = "foo")] pub mod foo;` to `lab-apis/src/lib.rs`
6. Add `foo = []` feature to `crates/lab-apis/Cargo.toml`
7. Create MCP dispatch in `crates/lab/src/mcp/services/foo.rs`
8. Create CLI subcommands in `crates/lab/src/cli/foo.rs`
9. Register in `crates/lab/src/mcp/registry.rs` and `crates/lab/src/cli.rs`
10. Add `foo = ["lab-apis/foo"]` passthrough to `crates/lab/Cargo.toml`
11. Add to plugin metadata in `crates/lab/src/tui/metadata.rs`

### Auth

Use the `Auth` enum from `lab_apis::core`. Never hardcode auth handling in a service module.

```rust
use lab_apis::core::{Auth, HttpClient};

impl FooClient {
    pub fn new(base_url: &str, auth: Auth) -> Self {
        Self {
            http: HttpClient::new(base_url, auth),
        }
    }
}
```

### Config Loading

**`lab-apis` never reads files or env vars on its own.** Config loading lives entirely in `crates/lab/src/config.rs`. The library exposes optional `from_env()` helpers; the binary calls them.

Naming convention for env vars (read by `lab`, not `lab-apis`):
- `{SERVICE}_URL` — base URL
- `{SERVICE}_API_KEY` — API key (for ApiKey auth)
- `{SERVICE}_TOKEN` — token (for Token/Bearer auth)
- `{SERVICE}_USERNAME` / `{SERVICE}_PASSWORD` — credentials (for Basic auth)

**Multi-instance services:** append a label before the suffix — `UNRAID_URL` is the default instance, `UNRAID_NODE2_URL` / `UNRAID_NODE2_API_KEY` is an additional named instance `node2`. MCP callers select via `params.instance`; CLI selects via `--instance` or positional label. Never hardcode instance names — derive them from env at startup.

Loaded from `~/.lab/.env`. **`extract.apply` writes to this file** using a strict merge algorithm (backup first, atomic write, dedupe by key, preserve order and comments, default conflict policy is skip-and-warn, `--force` overwrites). See `crates/lab-apis/src/extract/CLAUDE.md`.

### PluginMeta shape

Every service entry-point file (e.g., `radarr.rs`) declares a `pub const META: PluginMeta` with:

- `category: Category` — one of 10 variants: `Media`, `Servarr`, `Indexer`, `Download`, `Notes`, `Documents`, `Network`, `Notifications`, `Ai`, `Bootstrap`.
- `required_env: &[EnvVar]` / `optional_env: &[EnvVar]` — each `EnvVar { name, description, example, secret }`. `secret: true` marks values to mask in TUI/logs.
- `default_port: Option<u16>` — used by `lab doctor` and the TUI for hints.

### Error Handling

- `lab-apis`: use `thiserror` for typed errors per service; every service error wraps `ApiError` transparently.
- `lab` binary: use `anyhow` to wrap everything.
- Always return `Result<T>`, never panic.
- **`ApiError::kind()` taxonomy** (stable string tags consumed by MCP envelopes): `auth_failed`, `not_found`, `rate_limited`, `validation_failed`, `network_error`, `server_error`, `decode_error`, `internal_error`. Dispatchers add `unknown_action`, `unknown_subaction`, `missing_param`, `invalid_param`, `unknown_instance` on top. Never invent new kinds without updating DESIGN.md.

### Async trait style

Use **native `async fn in trait`** (stable in Rust 1.75+). Do **not** add the `async-trait` crate. Do **not** use `Box<dyn ServiceClient>` — prefer generics or concrete types. This is a hard rule; PRs that reintroduce `#[async_trait]` will be rejected.

### Output Formatting

All formatting lives in `crates/lab/src/output.rs`. `lab-apis` types are pure data.

- Derive `Tabled` on wrapper types in `lab` (not on `lab-apis` types — keeps `tabled` out of the SDK)
- Support `--json` by serializing the underlying `lab-apis` type with `serde_json`
- Use `tracing` for debug/verbose output, never `println!` for debug info

## Tech Stack

| Crate | Purpose | Lives in |
|-------|---------|----------|
| tokio | async runtime | both |
| reqwest | HTTP client (rustls-tls) | lab-apis |
| serde + serde_json | serialization | lab-apis |
| thiserror | library errors | lab-apis |
| wiremock | HTTP mocking (tests) | lab-apis |
| clap | CLI parsing (derive) | lab |
| rmcp | MCP server | lab |
| ratatui + crossterm | TUI | lab |
| tabled | table rendering | lab |
| dotenvy | .env loading | lab |
| toml | config parsing | lab |
| tracing | structured logging | lab |
| anyhow | binary errors | lab |

## Dev Commands

```bash
just check      # cargo check --workspace
just test       # cargo nextest run
just lint       # cargo clippy + cargo fmt --check
just deny       # cargo deny check
just build      # cargo build --features all
just run        # cargo run --features all -- <args>
just release    # cargo release
just mcp-token  # rotate the MCP bearer token in ~/.lab/.env
```

### Operator tooling

- **`lab doctor`** — comprehensive health audit: checks env vars, reachability, auth, version for every enabled service. Emits human-readable table by default, `--json` for CI. Exit code reflects worst severity.
- **`bin/health-check`** — repo-level shell helper for CI/CD smoke tests.

Scoped to a single crate:

```bash
cargo test -p lab-apis        # client tests only (fast, wiremock-based)
cargo test -p lab             # CLI/MCP/TUI tests only
```

## Testing

- Unit tests: mock HTTP with `wiremock` in `lab-apis`, run in CI
- Integration tests: hit real services, run locally only (marked `#[ignore]`)
- Test runner: `cargo-nextest` (parallel execution)

```bash
# Unit tests (CI-safe)
just test

# Integration tests (requires running services)
just test-integration
```

## CI

- GitHub Actions
- Matrix: linux x86_64, windows x86_64
- Checks: clippy, rustfmt, cargo-deny, nextest
- Release: cargo-release → GitHub Releases with pre-built binaries (linux x86_64, linux aarch64, windows x86_64)

## Style

- Rust 2024 edition, latest stable toolchain
- `cargo fmt` with default settings
- `cargo clippy` with no allowed warnings
- Prefer `impl Trait` over `Box<dyn Trait>` where possible
- Prefer concrete types over generics unless sharing demands it
- Never add `clap`, `rmcp`, `ratatui`, `anyhow`, or `tabled` to `lab-apis` — they belong in `lab` only
- **No `mod.rs` files.** Modern Rust module style only: a module `foo` is declared in `foo.rs` sibling to its `foo/` directory, never in `foo/mod.rs`
