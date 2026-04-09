# lab-apis

Pure Rust SDK for homelab service APIs. **Zero binary dependencies** — reusable in any Rust project.

```
tokio | reqwest | serde | thiserror
```

## What It Is

HTTP client library for 21 homelab services: Radarr, Sonarr, Prowlarr, Plex, qBittorrent, UniFi, Unraid, Overseerr, Tailscale, and others. Each service exposes a typed async client with request/response types and structured error handling.

Designed to be a library — not a binary, not an MCP server. Use it in your own Rust projects.

## Installation

```toml
[dependencies]
lab-apis = { version = "0.6", features = ["radarr", "sonarr"] }

# or all services:
lab-apis = { version = "0.6", features = ["all"] }
```

## Quick Start

```rust
use lab_apis::radarr::RadarrClient;
use lab_apis::core::Auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RadarrClient::new(
        "http://localhost:7878",
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "your-api-key".into(),
        },
    )?;

    let movies = client.get_movies().await?;
    println!("Found {} movies", movies.len());
    Ok(())
}
```

## Core Abstractions

### HttpClient
Shared HTTP wrapper around `reqwest::Client`. Hardcoded timeouts: 5s connect, 30s request. TLS via rustls. Fallible constructor — TLS init failure returns `ApiError::Internal`.

### Auth Enum
```rust
pub enum Auth {
    None,
    ApiKey { header: String, key: String },
    Token { token: String },
    Bearer { token: String },
    Basic { username: String, password: String },
    Session { cookie: String },
}
```

Header name for `ApiKey` is caller-chosen. Most *arr services use `X-Api-Key`.

### ServiceClient Trait
Every service implements a health check:
```rust
pub trait ServiceClient: Send + Sync {
    fn name(&self) -> &'static str;
    fn service_type(&self) -> &'static str;
    async fn health(&self) -> Result<ServiceStatus, ApiError>;
}
```

### Error Handling
All errors wrap to `ApiError` with stable `kind()` tags for programmatic handling:
- `auth_failed` — auth headers rejected
- `not_found` — 404
- `rate_limited` — 429, includes `retry_after` if available
- `validation_failed` — request validation error
- `network_error` — connectivity issue
- `server_error` — 5xx response
- `decode_error` — response parsing failed
- `internal_error` — unhandled error

See `docs/ERRORS.md` in the monorepo for the canonical error vocabulary.

## Service Status

| Status | Services |
|--------|----------|
| **Fully Implemented** | Radarr, UniFi, ByteStash |
| **Partially Implemented** | OpenAI (Chat, Embeddings), Overseerr (search, request management) |
| **Client Stub** | Sonarr, Prowlarr, Plex, Tautulli, SABnzbd, qBittorrent, Tailscale, Linkding, Memos, Paperless, Arcane, Gotify, Qdrant, TEI, Apprise |
| **Extract** | Service credential scanner — architecture present, transport/parsing in progress |

"Stub" means the client struct exists and compiles, but methods are minimal or unimplemented. Contributions welcome.

## Module Structure

Every service follows:
```
foo.rs              # pub const META: PluginMeta, ServiceClient impl
foo/
  client.rs         # FooClient struct and async methods
  types.rs          # Request/response types (serde)
  error.rs          # Service-specific error enum (thiserror)
```

No `mod.rs` files — modern Rust 1.56+ module style.

Large clients (Radarr, OpenAI) organize methods into sub-modules. Each `impl RadarrClient { ... }` block is a separate file under `client/` — methods accumulate across files without a monolithic `client.rs`.

## Metadata

Every service exports `pub const META: PluginMeta`:
```rust
pub struct PluginMeta {
    pub name: &'static str,
    pub category: Category,
    pub required_env: &'static [EnvVar],
    pub optional_env: &'static [EnvVar],
    pub default_port: Option<u16>,
}

pub struct EnvVar {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub secret: bool,  // true = mask in logs/TUI
}
```

Categories: `Media`, `Servarr`, `Indexer`, `Download`, `Notes`, `Documents`, `Network`, `Notifications`, `Ai`, `Bootstrap`.

## Config Loading

**`lab-apis` never reads files or environment variables.** Config lives entirely in the binary (`lab/src/config.rs`). The library exposes `Auth::from_env()` helpers; the binary calls them.

Standard env var naming:
- `{SERVICE}_URL` — base URL
- `{SERVICE}_API_KEY` — API key
- `{SERVICE}_TOKEN` — bearer token
- `{SERVICE}_USERNAME` / `{SERVICE}_PASSWORD` — Basic auth

Multi-instance services append a label: `RADARR_URL` (default), `RADARR_NODE2_URL` (instance `node2`).

## Feature Flags

21 opt-in features. `core` and `extract` always compile.

```rust
radarr, sonarr, prowlarr     // pulls in shared "servarr" types automatically
overseerr, plex, tautulli, sabnzbd, qbittorrent, tailscale
linkding, memos, bytestash, paperless, arcane, unraid, unifi
gotify, openai, qdrant, tei, apprise
```

Convenience: `features = ["all"]` enables all 21.

## Testing

Unit tests use `wiremock` for HTTP mocking:
```bash
cargo test -p lab-apis
```

Integration tests (marked `#[ignore]`) require live services:
```bash
cargo test -p lab-apis -- --ignored --nocapture
```

## Invariants

- **No `clap`, `rmcp`, `ratatui`, `anyhow`, `tabled`** — they belong in the `lab` binary only
- **No file or env I/O** — `Auth::from_env()` accepts values; the caller reads them
- **Native async fn in trait** (Rust 1.75+) — no `#[async_trait]`, no `Box<dyn>`
- **Debug impls redact secrets** — test this for any new auth variant
- **TLS is always rustls** — hardcoded, not configurable

## Architecture

For full context, see the monorepo docs:
- `docs/README.md` — architecture index
- `docs/ERRORS.md` — error taxonomy and stability guarantees
- `docs/OBSERVABILITY.md` — logging and tracing rules
- `docs/SERIALIZATION.md` — serde ownership and output boundaries
- `CLAUDE.md` (this directory) — per-crate development rules

## License

Same as the monorepo. See `LICENSE` at the repository root.
