# lab-apis — Pure SDK Crate

HTTP client library for all 21 homelab services. No binary dependencies (`clap`, `rmcp`,
`ratatui`, `anyhow`, `tabled` are forbidden here — they live in `lab` only).

Sub-docs for key sub-modules:
- [`src/core/CLAUDE.md`](src/core/CLAUDE.md) — HttpClient, Auth, traits, error taxonomy
- [`src/servarr/CLAUDE.md`](src/servarr/CLAUDE.md) — shared *arr primitives
- [`src/extract/CLAUDE.md`](src/extract/CLAUDE.md) — synthetic bootstrap service rules

## Feature Flags

21 opt-in features. `core` and `extract` are always compiled — no gate possible.

- `servarr` is pulled in **transitively** by `radarr`, `sonarr`, `prowlarr` — do not list it
  in `all` directly.
- `test-utils` is a reserved marker flag; nothing in the crate is currently gated on it.
- Default features are set by the `lab` binary crate, not here.

## Module Structure

Every service follows: `foo.rs` (module declaration + `META` + `ServiceClient` impl) alongside
`foo/` (client.rs, types.rs, error.rs). No `mod.rs` files anywhere.

When a service grows large, `client.rs` becomes a module declaration file with `pub mod`
sub-modules (`client/chat.rs`, etc.) — same pattern applied recursively. `openai` and
`overseerr` are the current examples.

**`ServiceClient` impl lives in `foo.rs`, not `client.rs`.** `META` lives there too.

## HttpClient Invariants

- `HttpClient::new()` is **fallible** — TLS init failure yields `ApiError::Internal`. All
  service `Client::new()` constructors must propagate `Result`.
- **connect_timeout: 5 s, request_timeout: 30 s** — hardcoded, not per-client configurable.
- **No retry logic, no backoff, no tracing spans inside `HttpClient`.** Callers own spans.
- `RateLimited.retry_after` is always `None` from `HttpClient`. Parse `Retry-After` manually
  if the value is needed.
- Absolute URL guard (`debug_assert!`) is **debug-only** — does not enforce in release builds.

## Auth Enum

| Variant | Wire |
|---------|------|
| `None` | nothing |
| `ApiKey { header, key }` | `<header>: <key>` (header name is caller-chosen) |
| `Token { token }` | `Authorization: Token <token>` |
| `Bearer { token }` | `Authorization: Bearer <token>` |
| `Basic { username, password }` | HTTP Basic |
| `Session { cookie }` | `Cookie: <cookie>` |

`Auth` does not implement `Serialize`/`Deserialize`. `Debug` redacts all secret fields.

## ServiceClient Trait

```rust
fn health(&self) -> impl Future<Output = Result<ServiceStatus, ApiError>> + Send;
