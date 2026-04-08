# Lab Design Document

> Pluggable homelab CLI + MCP server SDK in Rust. One binary to rule your entire self-hosted stack.

> Archive note: the active topic docs now live in [README.md](./README.md), [ARCH.md](./ARCH.md), [TECH.md](./TECH.md), [MCP.md](./MCP.md), [SERVICES.md](./SERVICES.md), [CLI.md](./CLI.md), [TUI.md](./TUI.md), [CONFIG.md](./CONFIG.md), [CONVENTIONS.md](./CONVENTIONS.md), [EXTRACT.md](./EXTRACT.md), and [OPERATIONS.md](./OPERATIONS.md). This file remains the long-form design record and rationale archive.

## Overview

`lab` is a single Rust binary that serves as both a CLI tool and an MCP server for managing self-hosted homelab services. It replaces the bash skill scripts in `jmagar/claude-homelab` with a fast, type-safe, pluggable architecture.

Users can:
- Run CLI commands: `lab radarr search "The Matrix"`
- Serve MCP tools: `lab serve --services radarr plex sabnzbd --port 8400`
- Manage plugins via TUI: `lab plugins`
- Install plugins from CLI: `lab install prowlarr`
- Health check everything: `lab health`
- Validate setup: `lab doctor`

## Architecture

### Workspace Layout

Two crates. Clean split between **pure API clients** and **the binary that uses them**.

```
lab/
├── Cargo.toml                  # workspace root
├── CLAUDE.md                   # development instructions
├── docs/DESIGN.md              # this file
├── Justfile                    # dev task runner
├── deny.toml                   # cargo-deny config
├── crates/
│   ├── lab-apis/               # PURE: all service clients + core primitives
│   │   ├── Cargo.toml          # deps: reqwest, serde, thiserror, tokio, async-trait
│   │   └── src/
│   │       ├── lib.rs                # re-exports, feature gates
│   │       ├── core.rs               # `pub mod http; pub mod auth; ...`
│   │       ├── core/                 # shared primitives
│   │       │   ├── http.rs           # HttpClient
│   │       │   ├── auth.rs           # Auth enum
│   │       │   ├── config.rs         # optional from_env() helpers
│   │       │   ├── error.rs          # ApiError
│   │       │   ├── status.rs         # ServiceStatus
│   │       │   ├── action.rs         # ActionSpec / ParamSpec
│   │       │   ├── plugin.rs         # PluginMeta
│   │       │   └── traits.rs         # ServiceClient trait
│   │       ├── servarr.rs            # `pub mod client; pub mod types;`
│   │       ├── servarr/              # shared *arr primitives
│   │       │   ├── client.rs
│   │       │   └── types.rs
│   │       ├── radarr.rs             # gated by `radarr` feature; declares submodules + `pub const META`
│   │       ├── radarr/
│   │       │   ├── client.rs         # RadarrClient — all business logic
│   │       │   ├── types.rs          # serde request/response types
│   │       │   └── error.rs          # RadarrError (thiserror)
│   │       ├── sonarr.rs             # gated by `sonarr` feature
│   │       ├── sonarr/
│   │       ├── prowlarr.rs
│   │       ├── prowlarr/
│   │       ├── plex.rs
│   │       ├── plex/
│   │       └── ... (one .rs + dir per service)
│   │
│   └── lab/                          # BINARY: cli, mcp, tui, dispatch, main
│       ├── Cargo.toml                # deps: lab-apis, clap, rmcp, ratatui, anyhow, tabled, tracing, dotenvy
│       └── src/
│           ├── main.rs
│           ├── cli.rs                # `pub mod radarr; pub mod sonarr; ...` + top-level Command enum
│           ├── cli/                  # clap subcommands (thin shims)
│           │   ├── radarr.rs
│           │   ├── sonarr.rs
│           │   └── ...
│           ├── mcp.rs                # `pub mod registry; pub mod resources; pub mod error; pub mod services;`
│           ├── mcp/
│           │   ├── registry.rs       # runtime tool registration
│           │   ├── resources.rs      # action catalog as MCP resources
│           │   ├── envelope.rs       # ToolEnvelope
│           │   ├── error.rs          # structured JSON error helpers
│           │   ├── services.rs       # `pub mod radarr; pub mod sonarr; ...`
│           │   └── services/         # one dispatch module per service
│           │       ├── radarr.rs
│           │       ├── sonarr.rs
│           │       └── ...
│           ├── tui.rs
│           ├── tui/                  # ratatui plugin manager
│           ├── config.rs             # ~/.lab/.env + ~/.config/lab/config.toml loading
│           └── output.rs             # table/json formatting
```

**Modern Rust module style:** no `mod.rs` files anywhere in the workspace. A module `foo` with submodules is declared in `foo.rs` (sibling to the `foo/` directory), which contains `pub mod bar; pub mod baz;`. This is the Rust 2018+ idiom and what `rustfmt` / clippy expect.

### Why two crates?

The split mirrors the **Golden Rule** at the crate level:

- **`lab-apis`** — pure business logic. No `clap`, no `rmcp`, no `ratatui`, no `anyhow`. Just `reqwest` + `serde` + `thiserror` + `tokio`. Reusable as a standalone Rust SDK in any other binary (Tauri app, Discord bot, custom dashboard).
- **`lab`** — all UX. `clap`, `rmcp`, `ratatui`, `anyhow`, `tabled` only compile here.

You **cannot** accidentally put business logic in a CLI command or MCP dispatch, because `lab-apis` doesn't even know `clap` or `rmcp` exist. The compiler enforces the rule.

**`lab-apis` makes no assumptions about how it's configured.** It exposes optional `from_env()` helpers, but never *requires* them. Consumers wire credentials however they want — env vars, Tauri secure storage, Vault, hardcoded for tests.

### Reusing `lab-apis` standalone

```rust
// some other binary — e.g. a Discord bot
use lab_apis::{Auth, RadarrClient, PlexClient};

let radarr = RadarrClient::new("http://localhost:7878", Auth::api_key("X-Api-Key", "abc"));
let movies = radarr.search("The Matrix").await?;
```

Consumers pick only the services they need:

```toml
[dependencies]
lab-apis = { version = "0.1", default-features = false, features = ["plex", "tautulli"] }
```

### Core Trait Contract

Every service client implements a common trait from `lab_apis::core`:

```rust
#[async_trait]
pub trait ServiceClient: Send + Sync {
    fn name(&self) -> &str;
    fn service_type(&self) -> ServiceType;
    async fn health(&self) -> Result<ServiceStatus>;
}
```

Each client exposes **operations** (not raw HTTP calls):

```rust
impl RadarrClient {
    pub async fn search(&self, query: &str) -> Result<Vec<Movie>> { ... }
    pub async fn add(&self, tmdb_id: u64, opts: AddMovieOpts) -> Result<Movie> { ... }
    pub async fn queue(&self) -> Result<Vec<QueueItem>> { ... }
}
```

CLI and MCP consumers in `lab/` are **thin shims** over these operations:

```
CLI: parse clap args → call client method → format output (table/json)
MCP: parse action+params → call client method → serialize JSON response
```

Bug fixes and new features go in `lab-apis`. Both consumers get them for free.

### Auth Model

Unified `Auth` enum in `lab_apis::core::auth`:

```rust
pub enum Auth {
    ApiKey { header: String, key: String },        // *arr, Tautulli, Linkding
    Token { token: String },                        // Plex, Tailscale, Paperless
    Basic { username: String, password: String },   // qBittorrent (pre-session)
    Session { cookie: String },                     // qBittorrent (post-login)
    Bearer { token: String },                       // Memos, ByteStash
}
```

### Shared Contracts

Nine cross-cutting types every service touches. Specced upfront so we don't re-invent them per service.

#### 1. `ServiceStatus`

What `ServiceClient::health()` returns. Consumed by `lab health`, `lab doctor`, the TUI, and the `system.status` MCP action on every service.

```rust
// lab-apis/src/core/status.rs
pub struct ServiceStatus {
    pub reachable: bool,            // TCP/HTTP reached the host
    pub auth_ok: bool,              // credentials accepted
    pub version: Option<String>,    // server version, if exposed
    pub latency_ms: u64,            // round-trip for the health probe
    pub message: Option<String>,    // human-readable detail (error string, etc.)
}
```

Rules:
- `reachable = false` ⇒ `auth_ok = false`, `version = None`
- Never panic; network errors map to `reachable = false` + `message`
- Health probes have a hard 5s timeout regardless of `HttpClient` defaults

#### 2. ID Newtypes

Every service defines newtype wrappers for its identifiers. Prevents passing a Sonarr `SeriesId` to a Radarr method, prevents accidentally sending a TMDB id where an internal id is expected.

```rust
// lab-apis/src/radarr/types.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MovieId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TmdbId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityProfileId(pub u32);
```

Rules:
- `#[serde(transparent)]` so JSON wire format stays a bare integer
- `Copy` if `<= u64`, otherwise `Clone`
- Implement `From<u64>` / `Into<u64>` for ergonomics
- One `Display` impl per id type so log lines and tables look clean
- Never share id types across services (no `lab_apis::core::Id`)

MCP dispatch parses raw integers from `params` and constructs the newtype at the boundary, so MCP callers stay ignorant.

#### 3. Config Schema (typed struct)

`lab` loads config into a single typed struct. No stringly-typed `HashMap` lookups anywhere downstream.

```rust
// lab/src/config.rs
#[derive(Debug, Deserialize)]
pub struct LabConfig {
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub services: ServicesConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub default_format: OutputFormat,    // Table | Json | Yaml | Csv
    #[serde(default = "default_true")]
    pub color: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default = "default_mcp_port")]
    pub default_port: u16,
    #[serde(default)]
    pub default_services: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ServicesConfig {
    pub radarr: Option<RadarrConfig>,
    pub sonarr: Option<SonarrConfig>,
    // ... one Option<FooConfig> per service, all #[cfg(feature = "foo")]
}

#[derive(Debug, Deserialize)]
pub struct RadarrConfig {
    pub default_quality_profile: Option<QualityProfileId>,
    pub default_root_folder: Option<PathBuf>,
}
```

Rules:
- Every field has a `serde` default — partial config files are valid
- Validation happens once at load (`LabConfig::load() -> Result<Self>`)
- Secrets (`*_URL`, `*_API_KEY`, etc.) come from `~/.lab/.env`, **not** the typed struct — they live in env vars and are read at client construction time
- Per-service config blocks use the service's own newtypes (e.g. `QualityProfileId`) so the type system enforces consistency

#### 4. MCP Tool Result Envelope

Every MCP tool call returns the same shape. Success and error are distinguishable without parsing strings.

**Success:**
```jsonc
{
  "ok": true,
  "service": "radarr",
  "action": "movie.search",
  "data": [ /* serialized lab-apis type */ ],
  "meta": {
    "latency_ms": 142,
    "count": 7          // present for list responses
  }
}
```

**Error:**
```jsonc
{
  "ok": false,
  "service": "radarr",
  "action": "movie.search",
  "error": {
    "kind": "auth_failed",       // canonical error kind, see below
    "message": "401 Unauthorized",
    "valid": null,               // populated for unknown_action errors
    "hint": null,                // populated when we can suggest a fix
    "retry_after_ms": null       // populated for rate_limited errors
  }
}
```

**Canonical `kind` values** (cross-service vocabulary):
`unknown_action`, `unknown_subaction`, `missing_param`, `invalid_param`,
`auth_failed`, `not_found`, `rate_limited`, `validation_failed`,
`network_error`, `server_error`, `decode_error`, `internal_error`.

The envelope is wrapped in MCP's standard `content` block as a single JSON text block. We do not use multiple content blocks per call — single JSON keeps parsing trivial for agents.

```rust
// lab/src/mcp/envelope.rs
pub struct ToolEnvelope<T: Serialize> { /* fields above */ }

impl<T: Serialize> ToolEnvelope<T> {
    pub fn success(service: &str, action: &str, data: T, latency: Duration) -> Self { ... }
    pub fn error(service: &str, action: &str, err: ToolError) -> Self { ... }
    pub fn into_mcp_content(self) -> Content { ... }
}
```

#### 5. Logging Shape

One standard `tracing` span per service call, emitted by `lab_apis::core::HttpClient`. Every log line gets the same structured fields, making logs greppable and machine-parseable.

```rust
// lab-apis/src/core/http.rs
let span = tracing::info_span!(
    "service_call",
    service = %self.service_name,    // "radarr"
    method  = %req.method(),          // "GET"
    path    = %req.url().path(),      // "/api/v3/movie"
    status  = tracing::field::Empty,  // filled on response
    latency_ms = tracing::field::Empty,
    error   = tracing::field::Empty,
);
```

The `lab` binary adds a parent span identifying the *caller* (CLI vs MCP):

```rust
// CLI: lab/src/cli/radarr.rs
let _g = tracing::info_span!("cli", command = "radarr.movie.search").entered();

// MCP: lab/src/mcp/services/radarr.rs
let _g = tracing::info_span!("mcp", action = %action).entered();
```

Resulting log line (JSON format):
```jsonc
{
  "ts": "2026-04-07T14:22:09Z",
  "level": "INFO",
  "target": "lab_apis::core::http",
  "span": "service_call",
  "service": "radarr",
  "method": "GET",
  "path": "/api/v3/movie",
  "status": 200,
  "latency_ms": 142,
  "parent_span": "mcp",
  "parent_action": "movie.search"
}
```

Subscriber config (in `lab/src/main.rs`):
- `LAB_LOG` env var controls filter (defaults to `lab=info,lab_apis=warn`)
- `--log-format=json|pretty` flag picks format (defaults to `pretty` for TTY, `json` otherwise)
- `--log-file <path>` optionally tees to a file
- HTTP request bodies are **never** logged (may contain credentials); response bodies only at `trace`

`lab doctor` consumes these spans to produce per-service health summaries (call count, p50/p99 latency, error rate) by attaching a custom `tracing` layer that aggregates `service_call` events.

#### 6. Error Taxonomy (`ApiError`)

A canonical error enum every service maps into. Service-specific errors wrap it. MCP envelope `kind` strings come from this enum's `Display`.

```rust
// lab-apis/src/core/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentication failed")]
    Auth,

    #[error("not found")]
    NotFound,

    #[error("rate limited")]
    RateLimited { retry_after: Option<Duration> },

    #[error("validation failed: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("server error {status}: {body}")]
    Server { status: u16, body: String },

    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("internal: {0}")]
    Internal(String),
}

impl ApiError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Auth          => "auth_failed",
            Self::NotFound      => "not_found",
            Self::RateLimited{..}=>"rate_limited",
            Self::Validation{..}=> "validation_failed",
            Self::Network(_)    => "network_error",
            Self::Server{..}    => "server_error",
            Self::Decode(_)     => "decode_error",
            Self::Internal(_)   => "internal_error",
        }
    }
}
```

Per-service errors:
```rust
// lab-apis/src/radarr/error.rs
#[derive(Debug, thiserror::Error)]
pub enum RadarrError {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("movie not in library: {0}")]
    MovieNotInLibrary(MovieId),

    #[error("quality profile {0} does not exist")]
    UnknownQualityProfile(QualityProfileId),
}
```

Rules:
- `HttpClient` returns `Result<T, ApiError>` and never anything else
- Service clients return `Result<T, FooError>`; `FooError: From<ApiError>`
- HTTP status → `ApiError` mapping is centralized in `HttpClient`:
  - `401`/`403` → `Auth`
  - `404` → `NotFound`
  - `429` → `RateLimited` (parses `Retry-After` header)
  - `4xx` with JSON body → `Validation` if parseable, else `Server`
  - `5xx` → `Server`
- MCP dispatch maps `FooError` → `ToolEnvelope::error` using `kind()`

#### 7. Action Catalog Schema (`ActionSpec`)

The single source of truth for: dispatch validation, `help` action responses, and `lab://service/actions` MCP resources. Defined once per service as a `static` slice; the dispatch macro reads it.

```rust
// lab-apis/src/core/action.rs
pub struct ActionSpec {
    pub name: &'static str,            // "movie.search"
    pub description: &'static str,
    pub destructive: bool,             // drives elicitation + CLI confirm
    pub params: &'static [ParamSpec],
    pub returns: &'static str,         // type name hint, e.g. "Movie[]"
}

pub struct ParamSpec {
    pub name: &'static str,
    pub ty: &'static str,              // "string" | "integer" | "number" | "boolean"
                                       // | "object" | "array" | "string[]" | "integer[]"
                                       // | "string|null" | enum literal "queued|running|done"
    pub required: bool,
    pub description: &'static str,
}
```

`ty` is a free-form string label rather than an enum so adding new shapes (union types, enum literals, array-of-X) doesn't require a code change. The MCP layer translates it to JSON Schema in one helper. Same labels are echoed verbatim in `help` output for human readers.

Per-service catalog:
```rust
// lab/src/mcp/services/radarr.rs
pub static ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "movie.search",
        description: "Search Radarr's indexers for a movie by title.",
        destructive: false,
        params: &[
            ParamSpec { name: "query", ty: "string",  required: true,
                        description: "Movie title to search for" },
            ParamSpec { name: "year",  ty: "integer", required: false,
                        description: "Release year to disambiguate" },
        ],
        returns: "Movie[]",
    },
    ActionSpec {
        name: "movie.add",
        description: "Add a movie to Radarr by TMDB id.",
        destructive: false,
        params: &[
            ParamSpec { name: "tmdb_id",            ty: "integer", required: true,  description: "TMDB movie id" },
            ParamSpec { name: "quality_profile_id", ty: "integer", required: true,  description: "Radarr quality profile id" },
            ParamSpec { name: "root_folder",        ty: "string",  required: false, description: "Override default root folder" },
        ],
        returns: "Movie",
    },
    // ...
];
```

Rules:
- The dispatch `match` arms and `ACTIONS` live in the same file — drift is a code review concern, not a runtime concern
- `help` (no args) returns `ACTIONS` serialized
- `help` with `params.action = "movie.add"` returns one entry
- `lab://radarr/actions` resource serves the same JSON
- Param validation in dispatch can be auto-generated from `ParamSpec` (`missing_param` / `invalid_param` errors come from one helper, not 50 ad-hoc checks)
- `returns` is a hint for agents, not a runtime contract — keep it short and human-readable

#### 8. `HttpClient` Contract

The single HTTP entry point for every service. Defines retry, timeout, headers, rate limiting, and `ApiError` mapping in one place so no service re-invents it.

```rust
// lab-apis/src/core/http.rs
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: Url,
    auth: Auth,
    service_name: &'static str,
    config: HttpConfig,
}

pub struct HttpConfig {
    pub timeout: Duration,              // default 30s, override per call
    pub connect_timeout: Duration,      // default 5s
    pub max_retries: u32,               // default 2
    pub retry_backoff: Duration,        // default 250ms, doubled per retry
    pub user_agent: String,             // default "lab/<version>"
    pub rate_limit: Option<RateLimit>,  // None unless service requires it
}

pub struct RateLimit {
    pub requests: u32,
    pub per: Duration,
}
```

Behavior contract:
- **Auth injection** — `Auth` is applied automatically per request based on its variant (header for `ApiKey`/`Bearer`, query param when needed, cookie for `Session`, etc.)
- **Retries** — only on `network_error` and `5xx`. Never retry `4xx`. Never retry non-idempotent methods (`POST`/`PUT`/`PATCH`/`DELETE`) unless the caller opts in via `RequestOpts::idempotent(true)`
- **Backoff** — exponential with jitter. Honors `Retry-After` header on `429` and `503`
- **Rate limiting** — token bucket per `HttpClient` instance, opt-in via `HttpConfig::rate_limit`. Plex and Tautulli get one configured by default
- **Timeouts** — `connect_timeout` and total `timeout` are both enforced. Health probes use a hard 5s regardless
- **Tracing** — every request opens the `service_call` span (see Logging Shape). No request body logging, ever. Response body only at `trace` level
- **Error mapping** — all HTTP errors flow through one `map_response()` function that converts `reqwest::Response` → `Result<T, ApiError>` per the taxonomy rules above
- **JSON-only by default** — `get_json`, `post_json`, `delete_json` are the primary methods. Raw bytes (`get_bytes`) exist for file downloads (Paperless, ByteStash) but are explicit
- **No query string concatenation in service code** — use `RequestBuilder::query(&[(k, v)])`. Service modules never touch `Url` directly

Example service usage:
```rust
// lab-apis/src/radarr/client.rs
impl RadarrClient {
    pub async fn search(&self, query: &str) -> Result<Vec<Movie>, RadarrError> {
        Ok(self.http
            .get_json("/api/v3/movie/lookup", &[("term", query)])
            .await?)
    }
}
```
No retry logic, no error mapping, no auth handling, no logging — all of it lives in `HttpClient`.

#### 9. Plugin Metadata (`PluginMeta`)

The TUI plugin manager, `lab install`, `lab uninstall`, `lab doctor`, and the `.mcp.json` patcher all read from one static table per service. Defined alongside the service so adding a new one doesn't require touching a central registry file.

```rust
// lab-apis/src/core/plugin.rs
pub struct PluginMeta {
    pub name: &'static str,            // "radarr" — matches feature flag and CLI subcommand
    pub display_name: &'static str,    // "Radarr"
    pub description: &'static str,     // one-line summary
    pub category: Category,            // Media | Download | Indexer | Notes | Bookmarks | Documents | Network
    pub docs_url: &'static str,
    pub required_env: &'static [EnvVar],
    pub optional_env: &'static [EnvVar],
    pub default_port: Option<u16>,
}

pub struct EnvVar {
    pub name: &'static str,            // "RADARR_API_KEY"
    pub description: &'static str,
    pub example: &'static str,         // "abc123def456" — never a real value
    pub secret: bool,                  // true → never echo, mask in TUI
}

pub enum Category {
    Media,        // Plex, Tautulli, Overseerr
    Servarr,      // Radarr, Sonarr
    Indexer,      // Prowlarr
    Download,     // SABnzbd, qBittorrent
    Notes,        // Memos, Linkding, ByteStash
    Documents,    // Paperless
    Network,      // Tailscale, UniFi, Unraid, Arcane
    Notifications,// Gotify, Apprise
    Ai,           // OpenAI, Qdrant, TEI
    Bootstrap,    // extract, init, doctor
}
```

Per-service:
```rust
// lab-apis/src/radarr.rs
pub const META: PluginMeta = PluginMeta {
    name: "radarr",
    display_name: "Radarr",
    description: "Movie collection manager for Usenet and BitTorrent",
    category: Category::Media,
    docs_url: "https://radarr.video/",
    required_env: &[
        EnvVar { name: "RADARR_URL",     description: "Base URL of your Radarr instance",
                 example: "http://localhost:7878", secret: false },
        EnvVar { name: "RADARR_API_KEY", description: "API key from Settings > General",
                 example: "abc123...", secret: true },
    ],
    optional_env: &[],
    default_port: Some(7878),
};
```

Rules:
- `META` is a `pub const` on every service module, gated by the same feature flag as the client
- `lab/src/tui/metadata.rs` collects all `META` constants into a `Vec<&'static PluginMeta>` filtered by enabled features
- `lab install foo` checks every `required_env` is present, prompts for any that aren't (masking `secret: true` values), then writes them to `~/.lab/.env` and patches `.mcp.json`
- `lab doctor` reports per-plugin: env vars present, env vars valid format, reachability, auth check
- TUI groups services by `Category`

### Config & Credentials

Config loading lives in **`lab` (the binary)**, not `lab-apis`. The library never reads files or env vars on its own.

**Secrets** live in `~/.lab/.env`:
```env
RADARR_URL=http://localhost:7878
RADARR_API_KEY=abc123
PLEX_URL=http://localhost:32400
PLEX_TOKEN=xyz789
```

**Preferences** live in `~/.config/lab/config.toml`:
```toml
[output]
default_format = "table"  # table | json
color = true

[mcp]
default_port = 8400

[services.radarr]
default_quality_profile = 1
default_root_folder = "/movies"
```

Loading order (in `lab/src/config.rs`):
1. `~/.lab/.env` (secrets, via `dotenvy`)
2. `~/.config/lab/config.toml` (preferences, via `toml`)
3. Environment variables (override)
4. `--config` flag (override config.toml path)

#### Multi-Instance Services

A few services run as **multiple instances** in the same homelab (e.g. one Unraid box per physical server). The env-var schema scales to N instances without any service-specific code:

```env
# Single instance — implicit "default" label
UNRAID_URL=https://tower.local/graphql
UNRAID_API_KEY=...

# Additional named instances — arbitrary label between service prefix and field
UNRAID_SHART_URL=https://other.local/graphql
UNRAID_SHART_API_KEY=...

UNRAID_BASEMENT_URL=https://basement.local/graphql
UNRAID_BASEMENT_API_KEY=...
```

**Parsing rule** (in `lab/src/config.rs`):

For each service `S`:
- `S_URL` + `S_API_KEY` (or `_TOKEN` / `_USERNAME`+`_PASSWORD` per the service's auth model) define an instance named `default`.
- `S_<LABEL>_URL` + `S_<LABEL>_<credfield>` pairs define additional instances named `<label>` (lowercased). Label is any `[A-Z0-9_]+` segment between the service prefix and the field name.
- Labels are **never hardcoded** in `lab` source — they come entirely from the user's env. The config loader emits a `HashMap<String, ServiceConfig>` per service.

**Default instance resolution:**
1. The instance literally named `default` (from plain `S_URL`), if present.
2. Otherwise, if exactly one instance is configured, that one.
3. Otherwise, no default — operations require an explicit `--instance` / `instance` param.

**Surface integration:**

```bash
# CLI
lab unraid array status                       # uses default instance
lab unraid array status --instance shart      # explicit
lab unraid array status -i shart              # short form
lab unraid instances                          # list configured instances
```

```jsonc
// MCP — every action on a multi-instance service accepts `instance` in params
unraid({ "action": "array.status" })                              // default
unraid({ "action": "array.status", "params": { "instance": "shart" } })
unraid({ "action": "instances" })                                 // list
```

The `instance` param is added uniformly by the dispatcher — individual `ActionSpec`s don't need to declare it. Unknown labels return a structured `unknown_instance` error envelope listing the valid set, same shape as `unknown_action`.

**`lab-apis` side:**

`UnraidClient::new(url, auth)` is unchanged — one client per instance. The binary holds a `HashMap<String, UnraidClient>` and looks up by instance label at dispatch time. The library has zero awareness of "instances"; that concept lives entirely in the binary's config layer.

**Which services support multi-instance?**

Any of them, mechanically — the parsing rule is generic. Practically: Unraid is the obvious one; users might also run multiple Plex servers, multiple qBittorrent boxes, etc. The pattern is opt-in per user, not declared in code. If your env only has `RADARR_URL` you get a single-instance Radarr and never see the `--instance` flag.

**Future migration path:**

If a user ever hits 4+ instances of one service and the env vars get unwieldy, multi-instance config can move to `~/.config/lab/config.toml`:

```toml
[unraid.instances.tootie]
url     = "https://..."
api_key = "${UNRAID_TOOTIE_API_KEY}"   # secret stays in env
default = true

[unraid.instances.shart]
url     = "https://..."
api_key = "${UNRAID_SHART_API_KEY}"
```

`${VAR}` interpolation at load time keeps secrets out of TOML. The `--instance` surface stays identical, so this migration is invisible to CLI and MCP users. Not implemented in v0.1 — env-var pattern is the only one for now.

### Feature-Gated Services

Both crates expose matching features. `lab` re-exports `lab-apis` features so a single `--features radarr,plex` controls everything.

**`crates/lab-apis/Cargo.toml`:**
```toml
[features]
default = []
radarr      = []
sonarr      = []
prowlarr    = []
overseerr   = []
plex        = []
tautulli    = []
sabnzbd     = []
qbittorrent = []
tailscale   = []
linkding    = []
memos       = []
bytestash   = []
paperless   = []
arcane      = []
unraid      = []
unifi       = []
gotify      = []
openai      = []
qdrant      = []
tei         = []
apprise     = []
servarr     = []   # internal, enabled by radarr/sonarr/prowlarr
all = ["radarr", "sonarr", "prowlarr", "overseerr", "plex", "tautulli",
       "sabnzbd", "qbittorrent", "tailscale", "linkding",
       "memos", "bytestash", "paperless",
       "arcane", "unraid", "unifi",
       "gotify", "openai", "qdrant", "tei", "apprise"]
```

**`crates/lab/Cargo.toml`:** every `lab-apis` feature gets a 1:1 passthrough so a single `--features` flag controls both crates. The default set is the Servarr trio plus Plex/SAB/qBit — the minimum useful media-stack install.

```toml
[features]
default = ["radarr", "sonarr", "prowlarr", "plex", "sabnzbd", "qbittorrent"]

# media stack
radarr      = ["lab-apis/radarr"]
sonarr      = ["lab-apis/sonarr"]
prowlarr    = ["lab-apis/prowlarr"]
overseerr   = ["lab-apis/overseerr"]
plex        = ["lab-apis/plex"]
tautulli    = ["lab-apis/tautulli"]
sabnzbd     = ["lab-apis/sabnzbd"]
qbittorrent = ["lab-apis/qbittorrent"]

# network / infrastructure
tailscale   = ["lab-apis/tailscale"]
unraid      = ["lab-apis/unraid"]
unifi       = ["lab-apis/unifi"]
arcane      = ["lab-apis/arcane"]

# notes / docs / bookmarks
linkding    = ["lab-apis/linkding"]
memos       = ["lab-apis/memos"]
bytestash   = ["lab-apis/bytestash"]
paperless   = ["lab-apis/paperless"]

# notifications
gotify      = ["lab-apis/gotify"]
apprise     = ["lab-apis/apprise"]

# AI / vector / inference
openai      = ["lab-apis/openai"]
qdrant      = ["lab-apis/qdrant"]
tei         = ["lab-apis/tei"]

all         = ["lab-apis/all"]
```

### MCP Server (Runtime Plugin Selection)

```bash
# Stdio transport (default — for Claude Desktop / .mcp.json)
lab serve --services radarr plex sabnzbd

# HTTP transport (network-accessible)
lab serve --transport http --host 127.0.0.1 --port 8400 --services all
```

#### Transports

`lab serve` supports **two MCP transports**, selected by `--transport` (or `LAB_MCP_TRANSPORT`):

| Transport | Use case | Auth |
|-----------|----------|------|
| `stdio` (default) | Claude Desktop, `.mcp.json`, local agents | inherited from process |
| `http` | Remote agents, network-accessible MCP, multi-client | bearer token (`LAB_MCP_HTTP_TOKEN`, required) |

HTTP transport is built on `rmcp`'s SSE/Streamable-HTTP server. It always requires `LAB_MCP_HTTP_TOKEN` to be set — `lab` refuses to start an unauthenticated HTTP server. CORS origins are opt-in via `LAB_MCP_HTTP_CORS_ORIGINS` (comma-separated).

Same dispatch code, same tool registry, same action catalog — only the transport layer differs.

#### One Tool Per Service (action + subaction pattern)

Each service exposes **exactly one MCP tool** named after the service. Operations are dispatched via a flat dotted `action` string plus a free-form `params` object.

This keeps the tool list small (~20 tools max instead of hundreds), keeps the JSON schema tiny regardless of how many actions a service supports, and lets us add new actions without re-registering tools.

**Tool schema (identical for every service):**

```jsonc
{
  "name": "radarr",
  "description": "Radarr movie management. Call with action='help' to discover actions.",
  "inputSchema": {
    "type": "object",
    "required": ["action"],
    "properties": {
      "action": { "type": "string", "description": "e.g. 'movie.search', 'queue.list', 'help'" },
      "params": { "type": "object", "description": "Action-specific parameters" }
    }
  }
}
```

**Example calls:**

```jsonc
radarr({ "action": "movie.search", "params": { "query": "The Matrix", "year": 1999 } })
radarr({ "action": "movie.add",    "params": { "tmdb_id": 603, "quality_profile_id": 1 } })
radarr({ "action": "queue.list" })
radarr({ "action": "help" })           // returns full action catalog
radarr({ "action": "help", "params": { "action": "movie.add" } })  // detail for one action
```

**Action naming:** `<resource>.<verb>`, lowercase, dot-separated. Examples: `movie.search`, `movie.add`, `queue.list`, `system.status`, `library.refresh`.

#### Dispatch (in `lab/src/mcp/services/`)

`lab` owns all action dispatch. `lab-apis` stays pure — no `rmcp` dep.

```rust
// crates/lab/src/mcp/services/radarr.rs
use lab_apis::radarr::RadarrClient;

pub async fn dispatch(client: &RadarrClient, action: &str, params: Value) -> ToolResult {
    match action {
        "help"         => help(params),
        "movie.search" => to_result(client.search(parse(&params, "query")?).await),
        "movie.add"    => to_result(client.add(parse(&params, "tmdb_id")?, params.into()).await),
        "queue.list"   => to_result(client.queue().await),
        // ...
        unknown => Err(unknown_action("radarr", unknown)),
    }
}
```

#### Registry

`lab/src/mcp/registry.rs` registers one tool per enabled service at runtime based on `--services`:

```rust
pub fn register(service: &str, registry: &mut ToolRegistry) {
    match service {
        #[cfg(feature = "radarr")]
        "radarr" => registry.add_tool("radarr", services::radarr::dispatch),
        #[cfg(feature = "sonarr")]
        "sonarr" => registry.add_tool("sonarr", services::sonarr::dispatch),
        // ...
        _ => {}
    }
}
```

#### Discovery: `help` subaction + MCP resources

Every service tool supports a built-in `help` action for in-context discovery, **and** `lab` exposes the full action catalog as MCP resources for clients that prefer resource-based discovery.

```
lab://radarr/actions           # full action catalog (JSON)
lab://radarr/actions/movie.add # schema + description for one action
```

#### Top-Level `lab` Meta-Tool

In addition to the per-service tools, `lab serve` always registers a single **meta-tool** named `lab` that aggregates discovery across every service currently enabled in the running server. It's the entry point for agents that don't yet know what's available.

**Tool schema:**

```jsonc
{
  "name": "lab",
  "description": "Top-level lab discovery and status. Call action='help' for the full catalog.",
  "inputSchema": {
    "type": "object",
    "required": ["action"],
    "properties": {
      "action":  { "type": "string", "description": "help | services | status | version" },
      "params":  { "type": "object" }
    }
  }
}
```

**Actions:**

| Action | Params | Returns |
|--------|--------|---------|
| `help` | none | Full catalog: every enabled service + every action's name, description, params, destructive flag |
| `help` | `{ "service": "radarr" }` | Just that one service's catalog (delegates to `radarr.help`) |
| `help` | `{ "service": "radarr", "action": "movie.add" }` | One action's schema (delegates to `radarr.help` with action filter) |
| `services` | none | Compact list of enabled service names |
| `status` | none | `ServiceStatus` for every enabled service (parallel health probe) |
| `version` | none | `lab` binary version, MCP transport, enabled features |

**Auto-generated from `ActionSpec`:**

The catalog is **not hand-maintained**. At server startup, `register_meta_tool()` walks the tool registry, pulls each service's `&'static [ActionSpec]` (the same one used by per-service `help` and `lab://<service>/actions`), and assembles them into one document. Adding a new action to a service automatically appears in `lab.help` — no second place to update.

```rust
// crates/lab/src/mcp/meta.rs
pub fn build_catalog(registry: &ToolRegistry) -> Catalog {
    Catalog {
        lab_version: env!("CARGO_PKG_VERSION"),
        transport: registry.transport_kind(),
        services: registry
            .iter()
            .map(|(name, tool)| ServiceCatalog {
                name: name.to_string(),
                description: tool.description(),
                actions: tool.action_specs().to_vec(),  // already declared per service
            })
            .collect(),
    }
}
```

Because the catalog only references services that were actually registered (which depends on `--services` and feature flags), `lab.help` always reflects the *running* server, not the maximum possible build. A binary compiled with `--features all` but started with `lab serve --services radarr,plex` shows exactly two services in `lab.help`.

**Example response (`lab.help`):**

```jsonc
{
  "lab_version": "0.1.0",
  "transport": "stdio",
  "services": [
    {
      "name": "radarr",
      "description": "Radarr movie management",
      "actions": [
        { "name": "movie.search", "description": "Search by title",   "destructive": false, "params": [...] },
        { "name": "movie.add",    "description": "Add a movie",       "destructive": false, "params": [...] },
        { "name": "movie.delete", "description": "Delete a movie",    "destructive": true,  "params": [...] },
        { "name": "queue.list",   "description": "List download queue","destructive": false, "params": [] }
      ]
    },
    {
      "name": "plex",
      "description": "Plex media server",
      "actions": [ ... ]
    }
  ]
}
```

**Same surface, two access modes:**

| Want… | Use |
|-------|-----|
| Agent-driven discovery (call a tool, parse JSON) | `lab({"action":"help"})` |
| Drill into one service | `lab({"action":"help","params":{"service":"radarr"}})` or `radarr({"action":"help"})` |
| Client-driven browsing (resource list in the UI) | `lab://catalog` (top-level) or `lab://<service>/actions` (per service) |

The `lab://catalog` MCP resource serves the exact same JSON as `lab.help` for clients that prefer resources over tool calls.

#### Shared Catalog Module — One Builder, Three Surfaces

`build_catalog()` lives in `crates/lab/src/catalog.rs` and is the **single function** that produces the discovery document. Three callers consume it, all reading from the exact same `&'static [ActionSpec]` slices declared per service:

| Surface | Caller | Returns |
|---------|--------|---------|
| MCP tool | `lab.help` action handler in `mcp/meta.rs` | `Catalog` serialized as the tool result |
| MCP resource | `lab://catalog` resource handler in `mcp/resources.rs` | Same `Catalog`, served as a resource read |
| CLI | `lab help` subcommand in `cli/help.rs` | Same `Catalog`, rendered as a table or JSON |

```rust
// crates/lab/src/catalog.rs
pub fn build_catalog(registry: &ToolRegistry) -> Catalog { /* see meta.rs example above */ }
```

There is exactly one place to change discovery output. Adding a new field to `Catalog` updates all three surfaces in lockstep. Adding a new action to a service automatically appears in all three because they all walk the same `ActionSpec` slices. The `mcp/meta.rs`, `mcp/resources.rs`, and `cli/help.rs` files are each ~10 lines: parse args, call `build_catalog`, format output.

#### Errors

Unknown actions, missing params, and dispatch failures return **structured JSON errors** so agents can self-correct without a help round trip:

```jsonc
{
  "error": "unknown_action",
  "service": "radarr",
  "given": "movei.search",
  "valid": ["movie.search", "movie.add", "movie.list", "queue.list", "..."],
  "hint": "did you mean 'movie.search'?"
}
```

```jsonc
{
  "error": "missing_param",
  "action": "movie.add",
  "param": "tmdb_id",
  "expected": "integer"
}
```

```jsonc
{
  "error": "unknown_instance",
  "service": "unraid",
  "given": "tooty",
  "valid": ["default", "tootie", "shart"],
  "hint": "did you mean 'tootie'?"
}
```

#### Elicitation for Destructive Operations

Some actions are **destructive** — they delete data, stop containers, kill VMs, wipe arrays, or otherwise can't be undone with a follow-up call. For those, `lab` uses MCP **elicitation** to require explicit user confirmation before dispatching the underlying client method.

Every action declared in a service's `ActionSpec` has a `destructive: bool` flag. When the MCP server receives a call to a destructive action, it issues an `elicitation/create` request to the client per the MCP spec, waits for the user's response, and only proceeds on `accept`. `decline` or `cancel` returns a structured `elicitation_declined` error envelope.

**Destructive examples (non-exhaustive):**

| Service | Action | Why destructive |
|---------|--------|-----------------|
| arcane | `container.remove`, `stack.down`, `image.prune`, `volume.remove` | data loss / service downtime |
| unraid | `array.stop`, `array.start`, `vm.force_stop`, `share.delete`, `parity.cancel` | array integrity / VM data |
| qbittorrent | `torrent.delete` (with `delete_files=true`) | file deletion |
| sabnzbd | `queue.purge`, `history.delete` (`del_files=true`) | file deletion |
| radarr / sonarr | `movie.delete`, `series.delete` (with `delete_files=true`) | file deletion |
| paperless | `document.delete`, `document.bulk_edit` (delete) | data loss |
| unifi | `device.restart`, `device.forget`, `client.block` | network impact |

**Elicitation policy** (`LAB_MCP_ELICIT`):

| Mode | Behavior |
|------|----------|
| `always` | Prompt on every destructive call (safest) |
| `session` (default) | Prompt once per `(service, action)` per session, then remember |
| `never` | Auto-confirm — only for trusted automation; logged loudly |

The elicitation prompt always includes: service, action, params, and a one-line human-readable description from the `ActionSpec`. Example:

```jsonc
// elicitation/create
{
  "message": "arcane.container.remove will permanently delete container 'plex' (id: abc123). This cannot be undone. Continue?",
  "requestedSchema": {
    "type": "object",
    "properties": { "confirm": { "type": "boolean" } },
    "required": ["confirm"]
  }
}
```

Clients that don't support elicitation get an `elicitation_unsupported` error envelope with a hint to set `LAB_MCP_ELICIT=never` if they accept the risk. We never silently downgrade.

#### CLI Confirmation for Destructive Operations

The CLI applies the same destructive flags but uses **interactive prompts** (via `dialoguer`) instead of MCP elicitation. The behavior parallels the MCP policy:

```bash
$ lab arcane container.remove plex
⚠  arcane.container.remove will permanently delete container 'plex' (abc123).
   This cannot be undone.
Continue? [y/N]: _
```

| Flag | Behavior |
|------|----------|
| (default) | Interactive prompt; defaults to **No** |
| `-y` / `--yes` | Skip prompt, proceed |
| `--no-confirm` | Fail loudly if a destructive op is reached without `-y` (for scripts) |
| `--dry-run` | Show what would happen, exit without dispatching |

`LAB_CLI_CONFIRM` (`always` / `session` / `never`) sets the default policy. In non-interactive shells (`!is_terminal::is_terminal(stdin)`), `lab` refuses to run destructive ops without explicit `-y` or `LAB_CLI_CONFIRM=never` — same rule as MCP, no silent yes.

#### Single Source of Truth: `ActionSpec.destructive`

The destructive flag lives on the `ActionSpec` in `lab-apis::core::action`, alongside the rest of the action metadata. Both the MCP dispatcher and the CLI subcommand layer read it from the same place — there's exactly one list to update when a new destructive action is added.

```rust
ActionSpec {
    name: "container.remove",
    description: "Permanently delete a Docker container",
    destructive: true,
    params: &[ParamSpec { name: "id", ty: "string", required: true, .. }],
    returns: "RemoveResult",
}
```

### .mcp.json Integration

When a user installs a plugin via `lab install prowlarr`:
1. Check required env vars, prompt if missing
2. Append `prowlarr` to `--services` args in `.mcp.json`
3. Optionally restart the MCP server

```json
{
  "mcpServers": {
    "lab": {
      "command": "lab",
      "args": ["serve", "--services", "radarr", "sonarr", "prowlarr"],
      "env": {
        "LAB_ENV": "~/.lab/.env"
      }
    }
  }
}
```

Uninstalling removes the service name from the args array.

### Operator Tooling

Two repo-level helpers complement the binary. Neither ships in the release artifact — they live in the repo for developers and self-hosters.

**`bin/health-check`** — POSIX shell script that probes every configured service in `.env` using `curl` with the right auth header per service. Used as a smoke test before declaring a homelab "online". Multi-instance services are detected by reading `.env` directly (`grep -E '^UNRAID_[A-Z0-9]+_URL='`) so labels stay dynamic. Output is colorized when stdout is a TTY, plain otherwise. Exit code is non-zero if any reachable-but-configured service fails.

**`just mcp-token`** — Justfile recipe that generates a 32-byte hex token (`openssl rand -hex 32`) and writes it to `LAB_MCP_HTTP_TOKEN=` in `.env`, preserving every other line. Run on first checkout and any time the HTTP MCP transport credential needs to rotate. Idempotent in the sense that re-running just produces a new token — there is no "is it already set" guard, because rotation is the point.

### `lab doctor`

Operational health and config audit subcommand. Read-only — never writes. Exit code reflects worst severity found (0 ok, 1 warn, 2 fail).

```bash
lab doctor                       # everything: env, reachability, auth, version
lab doctor --service radarr      # just one
lab doctor --json                # machine-readable
lab doctor --quick               # skip auth+version probes, only env-var presence
```

Per service it checks:

| Check | Severity if missing |
|-------|---------------------|
| Required env vars present (`PluginMeta::required_env`) | fail |
| Optional env vars present | info |
| URL parses and resolves DNS | fail |
| TCP connect within 5s | fail |
| HTTP 2xx on health endpoint | fail |
| Auth accepted (401/403 → fail) | fail |
| Version reported by service | warn if absent |
| Reported version vs known-broken set | warn |

Output (table mode):

```
service     env   reach   auth   version           latency
radarr      ✓     ✓       ✓      5.10.4.9211       142ms
sonarr      ✓     ✓       ✓      4.0.9.2244        118ms
prowlarr    ✓     ✗ DNS   —      —                 —
unraid:tootie  ✓  ✓       ✓      7.0.1             203ms
unraid:shart   ✓  ✓       ✓      7.0.1             198ms
unifi       ✗ UNIFI_API_KEY missing
```

JSON mode emits one `ServiceStatus` per service (multi-instance services emit one entry per instance, named `<service>:<label>`), wrapped in:

```jsonc
{
  "lab_version": "0.1.0",
  "checked_at": "2026-04-08T19:22:09Z",
  "summary": { "ok": 12, "warn": 1, "fail": 2 },
  "services": [ /* ServiceStatus[] */ ]
}
```

`lab doctor` is the diagnostic counterpart to `lab.help`: where `help` answers "what *can* I do", doctor answers "what's *actually working* right now".

### TUI Plugin Manager

Built with `ratatui` + `crossterm` in `lab/src/tui/`. Always compiled in (no feature flag).

```
$ lab plugins

┌─ Lab Plugin Manager ───────────────────────────┐
│                                                 │
│  [x] radarr      Movie management    installed  │
│  [x] sonarr      TV management       installed  │
│  [ ] prowlarr    Indexer manager      available  │
│  [x] plex        Media server         installed  │
│  [ ] tautulli    Plex analytics       available  │
│  ...                                            │
│                                                 │
│  ↑↓ navigate  space toggle  enter apply  q quit │
└─────────────────────────────────────────────────┘
```

### CLI Subcommand Structure

```
lab
├── radarr           # Movie management
│   ├── search
│   ├── add
│   ├── queue
│   ├── list
│   └── status
├── sonarr           # TV management
│   ├── search
│   ├── add
│   ├── series
│   ├── queue
│   └── calendar
├── prowlarr         # Indexer management
│   ├── search
│   ├── indexers
│   └── stats
├── overseerr        # Media request frontend
│   ├── request
│   ├── search
│   └── status
├── plex             # Media server
│   ├── libraries
│   ├── search
│   ├── sessions
│   └── recently-added
├── tautulli         # Plex analytics
│   ├── activity
│   ├── history
│   ├── users
│   └── stats
├── sabnzbd          # Usenet downloads
│   ├── queue
│   ├── history
│   ├── pause
│   ├── resume
│   └── speed
├── qbittorrent      # Torrent client
│   ├── list
│   ├── add
│   ├── pause
│   ├── resume
│   └── remove
├── tailscale        # VPN
│   ├── devices
│   ├── status
│   └── routes
├── linkding         # Bookmarks
│   ├── search
│   ├── add
│   ├── tags
│   └── list
├── memos            # Notes
│   ├── create
│   ├── list
│   ├── search
│   └── tags
├── bytestash        # Code snippets
│   ├── list
│   ├── search
│   ├── get
│   └── create
├── paperless        # Documents
│   ├── search
│   ├── upload
│   ├── tags
│   └── correspondents
├── arcane           # Docker management UI
│   ├── containers
│   ├── images
│   ├── stacks
│   └── status
├── unraid           # Unraid server (GraphQL)
│   ├── array
│   ├── docker
│   ├── vms
│   ├── shares
│   └── status
├── unifi            # UniFi Network Application
│   ├── sites
│   ├── devices
│   ├── clients
│   └── status
├── gotify           # Push notifications
│   ├── send
│   ├── messages
│   ├── apps
│   └── clients
├── openai           # OpenAI API (+ OpenAI-compatible servers)
│   ├── chat
│   ├── models
│   ├── embeddings
│   └── images
├── qdrant           # Vector database
│   ├── collections
│   ├── points
│   ├── search
│   └── snapshots
├── tei              # HF Text Embeddings Inference
│   ├── embed
│   ├── rerank
│   ├── predict
│   └── info
├── apprise          # Universal notification dispatcher
│   ├── notify
│   ├── add
│   ├── get
│   └── del
├── serve            # MCP server mode
├── plugins          # TUI plugin manager
├── install          # Install a plugin (CLI)
├── uninstall        # Uninstall a plugin (CLI)
├── init             # First-run setup wizard
├── health           # Ping all configured services
├── doctor           # Validate env vars, connectivity, versions
├── completions      # Generate shell completions
└── self-update      # Update lab binary from GitHub releases
```

## Adding a New Service

Mechanical, all in obvious places:

1. `mkdir crates/lab-apis/src/foo/`
2. Create `client.rs`, `types.rs`, `error.rs` (hand-write or generate from OpenAPI)
3. Implement `ServiceClient` trait for health checks
4. Add `#[cfg(feature = "foo")] pub mod foo;` to `crates/lab-apis/src/lib.rs`
5. Add `foo = []` to `crates/lab-apis/Cargo.toml` features
6. Add MCP dispatch in `crates/lab/src/mcp/services/foo.rs`
7. Add CLI subcommands in `crates/lab/src/cli/foo.rs`
8. Register in `crates/lab/src/mcp/registry.rs` and `crates/lab/src/cli.rs`
9. Add `foo = ["lab-apis/foo"]` passthrough to `crates/lab/Cargo.toml`
10. Add to plugin metadata in `crates/lab/src/tui/metadata.rs`

## The `extract` Synthetic Service

`extract` is a **bootstrap utility** for pulling API keys and URLs out of existing service config files (Radarr `config.xml`, Sonarr `config.xml`, Prowlarr `config.xml`, etc.) and writing them into `~/.lab/.env`. It removes the manual copy/paste step of onboarding.

### Why a "synthetic service"

`extract` doesn't wrap a remote API — it reads files. But it's exposed through the **same shape** as the 21 real services: one `lab-apis` module, two thin handlers (CLI + MCP), one MCP tool with dotted actions, one CLI subcommand. Mechanically identical, just with a file-reading client instead of an HTTP client.

This keeps the architecture uniform — no special "meta tool" plumbing, no separate dispatch path. The dispatcher doesn't care that `extract.scan` reads XML instead of hitting `/api/v3/movie`.

### Not feature-gated

Unlike the 21 services, `extract` is **always compiled in**. It belongs to the same category as `init`, `doctor`, and `health`: utilities that should be present out of the box, no opt-in. SSH support (for remote `device:/path` URIs) is part of the default build — `russh` lives in `[workspace.dependencies]` and is always pulled.

### Module layout

```
crates/lab-apis/src/
├── extract.rs                # pub mod client; pub mod types; pub mod error; pub mod parsers; pub const META
└── extract/
    ├── client.rs             # ExtractClient — orchestration, URI parsing, fs/ssh dispatch
    ├── types.rs              # Uri, ExtractedCreds, ServiceCreds, ExtractReport
    ├── error.rs              # ExtractError (thiserror)
    ├── transport.rs          # LocalFs + Ssh (russh) — read_file abstraction
    └── parsers/
        ├── radarr.rs         # config.xml → ServiceCreds { url, api_key }
        ├── sonarr.rs
        ├── prowlarr.rs
        ├── lidarr.rs         # bonus: not a service yet, but easy to parse
        ├── readarr.rs        # ditto
        ├── bazarr.rs
        ├── overseerr.rs      # settings.json
        └── tautulli.rs       # config.ini
```

### URI Format

```
<host>:/<absolute/path>      # ssh — e.g. squirts:/mnt/appdata
/<absolute/path>             # local fs — e.g. /home/jmagar/appdata
~/<path>                     # local fs, expanded — e.g. ~/appdata
```

`<host>` is resolved through the user's `~/.ssh/config` (so aliases like `squirts`, `tower`, `unraid` Just Work). Auth is whatever ssh-agent already has loaded. Keys, certs, jump hosts — all handled by `russh` reading the same config file as `ssh`.

### Parsing strategy

Each `parsers/<app>.rs` knows two things:
1. **Where its config lives** under an appdata root: `radarr/config.xml`, `prowlarr/config.xml`, `overseerr/config/settings.json`, etc.
2. **How to extract creds** from that file: XML XPath, JSON path, or INI key.

The `ExtractClient` walks the appdata root, tries every parser, collects whatever succeeds. Missing apps are silently skipped; corrupt files become per-parser warnings in the `ExtractReport`, never a fatal error. Realistically the Arr-stack apps share the same `config.xml` shape (`Port`, `BindAddress`, `ApiKey`, `UrlBase`, `EnableSsl`), so most parsers are ~20 lines.

### MCP actions

```jsonc
extract({ "action": "scan",  "params": { "uri": "squirts:/mnt/appdata" } })
//  → { "found": ["radarr","sonarr","prowlarr","overseerr"], "creds": {...}, "warnings": [] }

extract({ "action": "apply", "params": {
    "uri": "squirts:/mnt/appdata",
    "services": ["radarr","sonarr","prowlarr"],   // optional filter
    "env_path": "~/.lab/.env"                      // optional override
}})
//  → { "written": ["RADARR_URL","RADARR_API_KEY", ...], "backed_up_to": "~/.lab/.env.bak.<ts>" }

extract({ "action": "diff",  "params": { "uri": "squirts:/mnt/appdata" } })
//  → shows what scan found vs what's currently in ~/.lab/.env (no writes)

extract({ "action": "help" })
```

`extract.apply` is **destructive** (writes `.env`, even with backup). It carries `destructive: true` in its `ActionSpec`, which routes it through the MCP elicitation flow and the CLI confirmation prompt.

#### `.env` write format

`extract.apply` writes `~/.lab/.env` (or `--env-path`) following these rules so existing files are merged, not clobbered:

1. **Backup first.** Copy the current file to `~/.lab/.env.bak.<unix_ts>` *before* any write. Backups are not garbage-collected — manual cleanup. If the target doesn't exist yet, no backup is created.
2. **Atomic write.** Write to a sibling tempfile (`.env.tmp.<pid>`) and `rename(2)` over the target. Same POSIX guarantees as the `.mcp.json` patcher (§.mcp.json Integration).
3. **Preserve order and comments.** Existing lines stay in place. Comments (`# ...`) and blank lines pass through untouched. New keys are appended at the end under a generated section header: `# --- added by extract <ts> ---`.
4. **Dedupe by key.** A `KEY=value` line is identified by its `KEY` (case-sensitive, leading `export ` allowed). Duplicate keys in the existing file are left alone — extract only touches the *first* occurrence and warns about the rest in the report.
5. **Conflict policy.** When extract finds a key that already exists with a *different* value:
   - **Default:** keep the existing value. Add the extracted value to `ExtractReport.warnings` as `{ kind: "conflict", key, existing, extracted }`. Nothing is overwritten silently.
   - **`--force` (CLI) or `params.force = true` (MCP):** overwrite in place. Old value is preserved in the backup file.
6. **Quoting.** Values containing whitespace, `#`, `$`, `"`, `'`, `\`, or shell metacharacters are wrapped in double quotes with `"` and `\` escaped. Values that are pure `[A-Za-z0-9_./-]+` are written bare. This matches what `dotenvy` reads back.
7. **No partial writes.** If parsing or writing fails after the backup is taken, the tempfile is unlinked and the original file is left untouched. The backup stays.
8. **Idempotent.** Running `extract.apply` twice in a row on the same input is a no-op on the second run (every key already matches).

### CLI

```bash
lab extract squirts:/mnt/appdata                # = scan, prints table
lab extract squirts:/mnt/appdata --apply        # writes ~/.lab/.env (prompts first)
lab extract squirts:/mnt/appdata --diff         # show changes vs current .env
lab extract /home/jmagar/appdata --json         # local path, JSON output
lab extract squirts:/mnt/appdata --apply -y     # skip confirmation (scripts)
lab extract squirts:/mnt/appdata --apply --dry-run    # show what would change
```

Top-level subcommand. Thin shim → `ExtractClient`.

### Why this is the right shape

- **Reusable** outside lab: `lab-apis::extract::ExtractClient::scan(uri)` works in any binary
- **Discoverable** via the same `help` action and `lab://extract/actions` MCP resource
- **Safe** by default: never writes without explicit `--apply` (CLI) or destructive elicitation (MCP)
- **Extensible**: adding a new parser is one file in `parsers/`, no plumbing changes anywhere

## Services

### With OpenAPI Specs

| Service | Spec Source | API Version |
|---------|------------|-------------|
| Radarr | `Radarr/Radarr/develop/src/Radarr.Api.V3/openapi.json` | v3, OpenAPI 3.0.1 |
| Sonarr | `Sonarr/Sonarr/develop/src/Sonarr.Api.V3/openapi.json` | v3, OpenAPI 3.0.1 |
| Prowlarr | `Prowlarr/Prowlarr/develop/src/Prowlarr.Api.V1/openapi.json` | v1, OpenAPI 3.0.1 |
| Overseerr | `api.overseerr.dev` (Redoc-served OpenAPI) | v1, OpenAPI 3.x |
| Plex | `LukeHagar/plex-api-spec` (community) + `developer.plex.tv` (official) | v1, OpenAPI 3.x |
| Tailscale | Downloadable from `tailscale.com/api` | v2, OpenAPI 3.x |
| Paperless-ngx | Served at `/api/schema/` on instances (drf-spectacular) | OpenAPI 3.x |
| Memos | `proto/gen/openapi.yaml` in usememos/memos repo | OpenAPI from protobuf |
| ByteStash | Served at `/api-docs` on instances (Swagger UI) | OpenAPI/Swagger |
| Arcane | `getarcane.app` — full OpenAPI YAML committed to repo | OpenAPI 3.x |
| Gotify | `github.com/gotify/server/blob/master/docs/spec.json` | v2, Swagger 2.0 |
| OpenAI | `github.com/openai/openai-openapi` (`manual_spec` branch) | v1, OpenAPI 3.x |
| Qdrant | `github.com/qdrant/qdrant/blob/master/docs/redoc/master/openapi.json` | v1, OpenAPI 3.x |
| HF TEI | `github.com/huggingface/text-embeddings-inference/blob/main/docs/openapi.json` | v1, OpenAPI 3.x |

### Without OpenAPI Specs (hand-write from docs)

| Service | API Reference |
|---------|---------------|
| Apprise | `github.com/caronc/apprise-api` — small Flask REST wrapper around the Apprise library; README is the spec |
| Tautulli | `github.com/Tautulli/Tautulli/wiki/Tautulli-API-Reference` — flat `?cmd=` RPC style |
| SABnzbd | `sabnzbd.org/wiki/configuration/4.5/api` — flat `?mode=` RPC style |
| qBittorrent | `github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)` — REST |
| Linkding | `linkding.link/api/` — clean REST, Django-based |
| Unraid | Unraid GraphQL API — `docs.unraid.net/API/` (GraphQL, not REST) |
| UniFi | UniFi Network Application local API — `/proxy/network/integration/v1/` on controller, X-API-KEY auth |

### Dropped (revisit later)

| Service | Reason |
|---------|--------|
| Radicale | CalDAV/CardDAV (WebDAV protocol), not REST |

## Tech Stack

| Concern | Choice | Crate |
|---------|--------|-------|
| Language | Rust 2024 edition, MSRV 1.85 (bumped quarterly) | both |
| Async runtime | tokio | both |
| Async trait style | native `async fn in trait` (Rust 1.75+) | both |
| HTTP client | reqwest (rustls-tls) | lab-apis |
| URL handling | `url::Url` (never `String`) | lab-apis |
| Date/time | jiff | lab-apis |
| Serialization | serde + serde_json | lab-apis |
| Library errors | thiserror | lab-apis |
| HTTP mocking (tests) | wiremock | lab-apis |
| SSH transport (extract) | russh + russh-keys (always compiled) | lab-apis |
| XML parsing (extract) | quick-xml | lab-apis |
| Snapshot tests | insta | lab-apis |
| Concurrent batch ops | futures + tokio (`buffer_unordered`) | lab-apis |
| CLI framework | clap (derive) | lab |
| MCP server | rmcp | lab |
| TUI | ratatui + crossterm | lab |
| Table rendering | tabled | lab |
| Color output | owo-colors | lab |
| TTY detection | is-terminal | lab |
| Progress bars | indicatif | lab |
| Config | dotenvy (.env) + toml (config.toml) | lab |
| Logging | tracing + tracing-subscriber | lab |
| Binary errors | anyhow | lab |
| Test runner | cargo-nextest | both |
| Task runner | just | — |
| CI | GitHub Actions | — |
| Cross-compile (aarch64) | `cross` | — |
| Linting | clippy (workspace lints) + rustfmt + cargo-deny | — |
| Release | cargo-release + git-cliff | — |
| Targets | linux x86_64, linux aarch64, windows x86_64 | — |
| License | MIT + Apache-2.0 dual | — |
| Telemetry | **none, ever** (no analytics, no phone-home, no crash reports) | — |

## Locked Conventions

These are decisions, not suggestions. Code reviews enforce them.

### Workspace `Cargo.toml`

Single source of truth for versions and lints. Both crates inherit via `.workspace = true`.

```toml
# /Cargo.toml
[workspace]
resolver = "3"
members  = ["crates/lab-apis", "crates/lab"]

[workspace.package]
version      = "0.1.0"        # single workspace version — both crates bump together
edition      = "2024"
rust-version = "1.85"         # MSRV, bumped quarterly
license      = "MIT OR Apache-2.0"
repository   = "https://github.com/jmagar/lab"
authors      = ["jmagar"]

[workspace.dependencies]
# runtime
tokio        = { version = "1", features = ["rt-multi-thread", "macros", "signal", "fs", "io-util"] }
futures      = "0.3"

# http / serde / errors
reqwest      = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "gzip", "stream"] }
url          = { version = "2", features = ["serde"] }
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
thiserror    = "2"
anyhow       = "1"

# time
jiff         = { version = "0.2", features = ["serde"] }

# logging
tracing             = "0.1"
tracing-subscriber  = { version = "0.3", features = ["env-filter", "json", "fmt"] }

# cli / output
clap         = { version = "4", features = ["derive", "env"] }
tabled       = "0.16"
owo-colors   = "4"
is-terminal  = "0.4"
indicatif    = "0.17"

# config
dotenvy      = "0.15"
toml         = "0.8"

# tui
ratatui      = "0.28"
crossterm    = "0.28"

# mcp
rmcp         = "0.1"

# extract (always compiled — synthetic service)
russh        = "0.45"   # ssh transport for remote appdata URIs
russh-keys   = "0.45"   # honors ~/.ssh/config and ssh-agent
quick-xml    = { version = "0.36", features = ["serialize"] }   # Servarr config.xml parsing

# tests
wiremock     = "0.6"
insta        = { version = "1", features = ["json", "yaml"] }

[workspace.lints.rust]
missing_docs                   = "warn"      # only enforced on lab-apis (see below)
unsafe_code                    = "forbid"
unreachable_pub                = "warn"

[workspace.lints.clippy]
pedantic     = { level = "warn", priority = -1 }
nursery      = { level = "warn", priority = -1 }
cargo        = { level = "warn", priority = -1 }
unwrap_used  = "warn"
expect_used  = "warn"
panic        = "warn"
todo         = "warn"
dbg_macro    = "warn"
print_stdout = "warn"           # use tracing or output module
print_stderr = "warn"

# allow some pedantic noise
module_name_repetitions = "allow"
must_use_candidate      = "allow"
missing_errors_doc      = "allow"   # opt-in per crate

[profile.release]
lto              = "thin"
codegen-units    = 1
strip            = "symbols"
panic            = "abort"
opt-level        = 3
debug            = false
incremental      = false

[profile.dev]
opt-level = 0
debug     = "line-tables-only"   # smaller debug info, faster link

[profile.dev.package."*"]
opt-level = 1                    # speed up dependency code in dev builds

[profile.release-debug]          # for profiling
inherits = "release"
debug    = true
strip    = "none"
```

Each crate's `Cargo.toml` inherits everything:

```toml
# crates/lab-apis/Cargo.toml
[package]
name         = "lab-apis"
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true

[dependencies]
tokio.workspace       = true
reqwest.workspace     = true
url.workspace         = true
serde.workspace       = true
serde_json.workspace  = true
thiserror.workspace   = true
jiff.workspace        = true
tracing.workspace     = true
futures.workspace     = true

[dev-dependencies]
wiremock.workspace = true
insta.workspace    = true

[features]
default = []
# ... see Feature-Gated Services
test-utils = []   # exposes mock builders for downstream tests

[lints]
workspace = true
```

### Async Trait Style — Native `async fn in trait`

We use **native** `async fn in trait` (stable since Rust 1.75). No `async-trait` crate, no `Box<dyn Future>`.

This works for our setup because the MCP `ToolRegistry` stores **function pointers** (`fn(&Client, &str, Value) -> impl Future`), not `Box<dyn ServiceClient>`. We never need a `dyn ServiceClient` trait object. Health checks are dispatched via a generated `match` over an enum of concrete client types, not via dynamic dispatch.

```rust
// lab-apis/src/core/traits.rs
pub trait ServiceClient: Send + Sync {
    fn name(&self) -> &str;
    fn service_type(&self) -> ServiceType;
    async fn health(&self) -> Result<ServiceStatus, ApiError>;  // native, no #[async_trait]
}
```

Forbidden patterns:
- `Box<dyn ServiceClient>` — forces async-trait, kills the optimization
- `&dyn ServiceClient` — same
- `Vec<Box<dyn ServiceClient>>` — use an enum dispatch instead

If we ever truly need a trait object, we add `async-trait` only on that one trait, never crate-wide.

### Cancellation — Top-Level `tokio::select!`, No Tokens in Method Signatures

No `CancellationToken` parameters anywhere in `lab-apis`. Long-running operations are cancelled by **dropping the future**, which `reqwest` and `tokio` handle correctly.

The binary installs one ctrl-c handler in `main.rs` and races every long-running command against it:

```rust
// crates/lab/src/main.rs
async fn run(cmd: Command) -> anyhow::Result<()> {
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    tokio::select! {
        biased;
        _ = &mut shutdown => {
            tracing::info!("shutdown requested");
            Ok(())
        }
        result = cmd.execute() => result,
    }
}
```

`lab serve` (MCP) gets the same treatment with a graceful drain: stop accepting new requests, let in-flight ones finish for up to 10s, then exit.

### Concurrent Batch Operations

**Convention:** every "create / add / delete" client method comes in two forms:

```rust
// single — the canonical, simple form
pub async fn add(&self, tmdb_id: TmdbId, opts: AddMovieOpts) -> Result<Movie, RadarrError>;

// batch — uses buffer_unordered with a default concurrency of 8
pub async fn add_many(
    &self,
    items: &[(TmdbId, AddMovieOpts)],
) -> Vec<Result<Movie, RadarrError>>;
```

Rules:
- Batch methods are named `<verb>_many`
- Default concurrency = 8 (configurable via `HttpConfig::batch_concurrency`)
- Batch methods return `Vec<Result<T, E>>`, **not** `Result<Vec<T>, E>` — partial failures are normal
- Only add a `_many` variant when there's a real use case (e.g. `radarr.add_many` yes, `radarr.system_status_many` no)
- The batch implementation is always:
  ```rust
  use futures::stream::{self, StreamExt};
  stream::iter(items)
      .map(|item| self.add(item.0.clone(), item.1.clone()))
      .buffer_unordered(self.http.config.batch_concurrency)
      .collect()
      .await
  ```

CLI batch commands accept multiple positional args (`lab radarr add 603 604 605`) and call `add_many` under the hood. MCP exposes `movie.add` (single) only — agents that want batch issue parallel tool calls, which is what they're good at.

### Progress Reporting

Long CLI ops use `indicatif` for progress bars, returned via an optional `ProgressSink` parameter on the ~5 client methods that need it (uploads, downloads, batch ops with >10 items).

```rust
// lab-apis/src/core/progress.rs
pub trait ProgressSink: Send + Sync {
    fn start(&self, total: Option<u64>, label: &str);
    fn advance(&self, delta: u64);
    fn finish(&self, message: &str);
}

// no-op default
impl ProgressSink for () { ... }
```

```rust
// lab-apis/src/paperless/client.rs
pub async fn upload(
    &self,
    path: &Path,
    progress: &dyn ProgressSink,
) -> Result<DocumentId, PaperlessError> { ... }
```

`lab/src/output.rs` provides an `IndicatifSink` that wraps `indicatif::ProgressBar` with the standard lab style:

```rust
// looks like:
//   Uploading bigfile.pdf  [████████████████░░░░░░░░] 67% (4.2 MB / 6.3 MB)  ETA 00:03
```

MCP calls always pass `&()` (no progress) — agents don't watch progress bars.

### Public API Surface (`lab-apis/src/lib.rs`)

Re-export the **client type** and **`META`** at the crate root for SDK ergonomics. Leave types and errors in submodules so the namespace stays navigable.

```rust
// lab-apis/src/lib.rs
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! `lab-apis` — typed async clients for self-hosted homelab services.
//!
//! Each service is feature-gated. Enable only what you need:
//! ```toml
//! lab-apis = { version = "0.1", default-features = false, features = ["radarr", "plex"] }
//! ```

pub mod core;

#[cfg(feature = "servarr")]    pub mod servarr;
#[cfg(feature = "radarr")]     pub mod radarr;
#[cfg(feature = "sonarr")]     pub mod sonarr;
#[cfg(feature = "prowlarr")]   pub mod prowlarr;
#[cfg(feature = "plex")]       pub mod plex;
#[cfg(feature = "tautulli")]   pub mod tautulli;
#[cfg(feature = "sabnzbd")]    pub mod sabnzbd;
#[cfg(feature = "qbittorrent")] pub mod qbittorrent;
#[cfg(feature = "tailscale")]  pub mod tailscale;
#[cfg(feature = "linkding")]   pub mod linkding;
#[cfg(feature = "memos")]      pub mod memos;
#[cfg(feature = "bytestash")]  pub mod bytestash;
#[cfg(feature = "paperless")]  pub mod paperless;
#[cfg(feature = "gotify")]     pub mod gotify;
#[cfg(feature = "openai")]     pub mod openai;
#[cfg(feature = "qdrant")]     pub mod qdrant;
#[cfg(feature = "tei")]        pub mod tei;
#[cfg(feature = "apprise")]    pub mod apprise;

// Flat re-exports — only the most-used types
pub use core::{
    Auth, ApiError, ServiceClient, ServiceStatus, ServiceType,
    HttpClient, HttpConfig, ActionSpec, ParamSpec, ParamType, PluginMeta,
};

#[cfg(feature = "radarr")]
#[doc(cfg(feature = "radarr"))]
pub use radarr::RadarrClient;

#[cfg(feature = "sonarr")]
#[doc(cfg(feature = "sonarr"))]
pub use sonarr::SonarrClient;

// ... one re-export per service client
```

Rules:
- Crate root re-exports: clients + `core` essentials only. Never types like `Movie` or `Series`.
- Errors stay in submodules: `lab_apis::radarr::RadarrError`, not `lab_apis::RadarrError`
- `META` is accessed as `lab_apis::radarr::META` — not flat-exported (would cause name collisions)
- `pub use` is for ergonomics, not for hiding the module structure — both forms work

### Documentation Policy (`lab-apis` only)

`lab-apis` is a real published SDK. It looks like one on docs.rs.

```rust
// lab-apis/src/lib.rs
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
```

Rules:
- Every `pub` item has a doc comment
- Every public client method has an `# Examples` section that compiles
- Every public type has a one-line summary on the first line, then a blank line, then detail
- Feature-gated items use `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]` so docs.rs renders the gate badge
- `cargo doc --no-deps --all-features` must succeed with zero warnings
- CI runs `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`

`lab` (the binary) is exempt — `#![allow(missing_docs)]` at its crate root. Binary internals don't need rustdoc.

### Testing Strategy

Three layers:

**1. Unit tests with `wiremock` (CI-safe, fast)**
```
crates/lab-apis/
├── src/
│   └── radarr/
│       └── client.rs        # tests inline in #[cfg(test)] mod tests
└── tests/
    └── fixtures/
        └── radarr/
            ├── movie_search.json     # captured real responses
            ├── movie_add.json
            └── queue_list.json
```

Inline tests use shared fixture loading from `lab_apis::test_utils`:

```rust
// crates/lab-apis/src/test_utils.rs (gated by `test-utils` feature)
pub fn fixture(service: &str, name: &str) -> String { ... }
pub fn mock_server() -> wiremock::MockServer { ... }
pub fn mock_radarr() -> (RadarrClient, MockServer) { ... }
```

**2. Snapshot tests with `insta` for serde parsing**
```rust
#[test]
fn parses_movie_search_response() {
    let raw = fixture("radarr", "movie_search.json");
    let parsed: Vec<Movie> = serde_json::from_str(&raw).unwrap();
    insta::assert_json_snapshot!(parsed);
}
```
Catches accidental wire-format breakage. Snapshots live in `crates/lab-apis/src/snapshots/`.

**3. Live integration tests against your real homelab**
```rust
// crates/lab-apis/tests/integration/radarr.rs
#[tokio::test]
#[ignore = "requires LAB_INTEGRATION=1 and live radarr"]
async fn live_radarr_search() {
    if std::env::var("LAB_INTEGRATION").is_err() { return; }
    let client = RadarrClient::from_env().unwrap();
    let movies = client.search("Inception", None).await.unwrap();
    assert!(!movies.is_empty());
}
```

Runs locally only:
```bash
LAB_INTEGRATION=1 cargo nextest run --features all -- --ignored
```

CI never runs live tests. Live tests **may** mutate state (you said you control the services), so each test cleans up after itself or uses a dedicated test tag/folder.

Test guards:
- All live tests are `#[ignore]` + check `LAB_INTEGRATION=1` env var
- One Justfile target: `just test-integration`
- Live tests live in `crates/lab-apis/tests/integration/<service>.rs`, never in `src/`

### Output Formatting

Lives in `crates/lab/src/output.rs`. Three formats: `Table` (default for TTY), `Json` (default for pipes), `Yaml` (opt-in).

```rust
pub enum OutputFormat { Table, Json, JsonCompact, Yaml }

impl OutputFormat {
    pub fn auto() -> Self {
        use is_terminal::IsTerminal;
        if std::io::stdout().is_terminal() { Self::Table } else { Self::JsonCompact }
    }
}
```

Rules:
- **JSON output:** pretty (`serde_json::to_string_pretty`) when stdout is a TTY, compact (`serde_json::to_string`) when piped. Never both at once.
- **Color:** `owo-colors` only. Auto-disabled when `!stdout().is_terminal()` or `NO_COLOR` env var is set or `--no-color` flag is passed.
- **Table rendering:** wrap `lab-apis` types in local `*Row` structs that derive `Tabled`. Never derive `Tabled` on `lab-apis` types — keeps `tabled` out of the SDK.
- **`println!` is banned** in `lab` (`clippy::print_stdout = "warn"`). Use the `output` module or `tracing`.

### Plugin Install / `.mcp.json` Patching

`lab install <service>` and `lab uninstall <service>` are atomic, backed up, and have a dry-run mode.

Behavior:
1. Locate `.mcp.json` (search `./`, `~/.config/claude/`, `~/Library/Application Support/Claude/`, etc., or `--mcp-json <path>`)
2. Parse it; if missing, create with the lab entry from scratch
3. Compute the new args array (`["serve", "--services", ...sorted_unique_services]`)
4. Diff the change; if `--dry-run`, print the diff and exit
5. Write `.mcp.json.bak` (overwriting any previous backup)
6. Atomically write `.mcp.json` via tempfile + `rename` in the same directory (POSIX atomic)
7. Verify the rewritten file parses; if not, restore from backup and error

```bash
lab install prowlarr           # patches .mcp.json
lab install prowlarr --dry-run # shows diff, no write
lab install prowlarr --mcp-json /custom/path/.mcp.json
lab uninstall prowlarr
```

Required env vars from the plugin's `META.required_env` are checked before patching. Missing values are prompted interactively (masking `secret: true` fields), then written to `~/.lab/.env` (also via atomic write + backup).

### Self-Update

`lab self-update` queries the GitHub releases API for `jmagar/lab`, downloads the binary for the current platform, verifies the SHA256 from the release notes, and replaces the running binary via `self_replace` (or equivalent).

- **No signature verification** (no sigstore, no minisign). Homelab scope. SHA256 from the release page is the only integrity check.
- **No automatic update checks** at startup. Update only when explicitly invoked.
- `--check` flag reports whether a newer version exists without installing.

### Secrets Handling

No `zeroize` on `Auth` fields. This is a homelab tool running as the user; the threat model doesn't include process memory inspection. Documented here so nobody adds it later "to be safe."

What we **do** care about:
- Never log credential values (enforced by `HttpClient` — request bodies are never logged at any level, headers with `Authorization` / `X-Api-Key` are masked)
- Never echo `secret: true` env vars in TUI prompts or `lab doctor` output
- Never write credentials to anywhere except `~/.lab/.env`

### TUI State

**Pure ephemeral.** No persistence. The TUI does not write to `~/.config/lab/tui-state.toml` or anywhere else. Last-viewed tab, scroll position, selection — all reset on every launch. Lock this in to prevent feature creep.

### Telemetry

**None. Ever.** No analytics, no phone-home, no crash reports, no opt-in metrics, no "anonymous usage data." Lab does not make outbound network requests except to:
1. Services explicitly configured by the user
2. `api.github.com` when `lab self-update` is explicitly invoked
3. `crates.io` / `docs.rs` are irrelevant (not made by the binary)

This is a hard rule. Any PR adding telemetry of any kind is rejected.

### CI

GitHub Actions, three jobs:

1. **`test`** — ubuntu-latest + windows-latest matrix
   - `cargo nextest run --workspace --all-features`
   - `cargo clippy --workspace --all-features -- -D warnings`
   - `cargo fmt --check`
   - `cargo deny check`
   - `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` (ubuntu only)
   - Cache via `Swatinem/rust-cache@v2`

2. **`build-release`** — runs on tag push, builds:
   - `x86_64-unknown-linux-gnu` (native on ubuntu)
   - `aarch64-unknown-linux-gnu` (via `cross`)
   - `x86_64-pc-windows-msvc` (native on windows)
   - Uploads to GitHub Releases via `softprops/action-gh-release`

3. **`publish`** — runs on tag push after `build-release` succeeds:
   - `cargo publish -p lab-apis`
   - `cargo publish -p lab` (5s sleep between to let crates.io index)

Live integration tests are **never** run in CI.

### Release Process

- **Single workspace version.** `lab` and `lab-apis` always bump together. The workspace version in the root `Cargo.toml` is the source of truth.
- **`cargo-release`** handles tagging and version bumps:
  ```bash
  cargo release patch    # 0.1.0 → 0.1.1
  cargo release minor    # 0.1.1 → 0.2.0
  ```
- **`git-cliff`** generates `CHANGELOG.md` from conventional commits before each release.
- **Tag format:** `v0.1.0` (lowercase `v` prefix).
- **`lab-apis` is published to crates.io from day one.** Even at 0.1.x, it's available for external consumers. SemVer rules apply: breaking changes bump the minor in 0.x, the major in 1.x+.

## Development Phases

### Phase 1: Foundation (serial, ~2-4 hours)
- `lab-apis::core`: HttpClient, Auth, errors, traits
- `lab-apis::servarr`: shared *arr types and base client
- `lab-apis::radarr`: first full service implementation (with wiremock tests)
- `lab` binary: radarr CLI subcommands + radarr MCP dispatch + `serve` + `health`
- End-to-end proof: CLI works, MCP serves, tests pass

### Phase 2: Validate Sharing (serial, ~1-2 hours)
- `lab-apis::sonarr` + `lab-apis::prowlarr`
- Confirm `servarr` module works cleanly
- Refine traits if needed

### Phase 3: Parallel Implementation (~15 min per service)
- All remaining services in parallel (up to 10 simultaneous)
- Each is independent: spec/docs in, working module out

### Phase 4: Polish (~2-3 hours)
- TUI plugin manager
- `.mcp.json` patching
- `lab init` wizard
- `lab doctor`
- `lab self-update`
- Shell completions
- CI/CD + release pipeline

## Integration with claude-homelab

`lab` fully replaces the bash skill scripts in `jmagar/claude-homelab`:

1. Skills' `SKILL.md` files reference `lab` commands instead of bash scripts
2. `scripts/` directories become unnecessary
3. `scripts/load-env.sh` replaced by lab's config loading
4. Single `.mcp.json` entry replaces multiple MCP server containers

```markdown
# Before (bash)
source "$REPO_ROOT/scripts/load-env.sh"
curl -s "$RADARR_URL/api/v3/movie?apikey=$RADARR_API_KEY" | jq '.[] | .title'

# After (lab)
lab radarr list
```
