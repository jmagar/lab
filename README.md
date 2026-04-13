# lab

`lab` is a Rust workspace for running a homelab control plane from one codebase: a reusable SDK in `lab-apis`, plus a `lab` binary that exposes the same services through a CLI, an MCP server, an HTTP API, and a TUI plugin manager.

The project is intentionally broad, but the shape is consistent: one service module per integration, one MCP tool per service, one shared action catalog, and one binary that can be filtered down to the services you actually want to expose.

## Current state

The repository is active and the wiring is ahead of full per-service implementation depth.

In the current `--all-features` build, the catalog registers **22 services** total: **21 feature-gated services** plus the always-on synthetic `extract` service. Of those, **10 currently expose callable actions**:

| Service | Status | Actions |
| --- | --- | ---: |
| `extract` | available | 5 |
| `radarr` | available | 41 |
| `sabnzbd` | available | 12 |
| `bytestash` | available | 18 |
| `unraid` | available | 10 |
| `unifi` | available | 72 |
| `gotify` | available | 13 |
| `linkding` | available | 13 |
| `paperless` | available | — |
| `prowlarr` | available | — |

The remaining compiled services are present in the catalog as **stubs** today: `sonarr`, `plex`, `tautulli`, `qbittorrent`, `tailscale`, `memos`, `arcane`, `overseerr`, `openai`, `qdrant`, `tei`, and `apprise`.

There are also a few repo-level gaps worth calling out up front:

- `lab install`, `lab uninstall`, and `lab init` are still stubbed.
- `lab extract --apply` and `lab extract --diff` are not finished yet.
- The TUI can already patch `.mcp.json`, but the matching CLI installer flow is still catching up.

## Workspace layout

| Path | Role |
| --- | --- |
| `crates/lab-apis` | Pure Rust SDK: typed clients, request/response models, auth, shared HTTP behavior, health contracts, plugin metadata |
| `crates/lab` | Product binary: CLI, MCP server, HTTP API, TUI, config loading, output rendering, discovery catalog |
| `docs/` | Topic-based source-of-truth documentation |

The main architectural rule is simple: **shared service logic belongs in `lab-apis`; the `lab` crate adapts that logic for user-facing surfaces**.

## What the binary exposes

### CLI

Top-level commands are defined in `crates/lab/src/cli.rs`:

- `lab help`
- `lab serve`
- `lab doctor`
- `lab health`
- `lab plugins`
- `lab completions`
- one subcommand per compiled service

`radarr` currently has the deepest CLI surface. Example commands:

```bash
lab help --json
lab doctor
lab health
lab radarr system-status
lab radarr movie-lookup "Inception"
lab extract /mnt/appdata
```

### MCP

`lab serve` starts the MCP server. The default transport is `stdio`; HTTP transport is also wired.

Every service is exposed as **one MCP tool** that accepts a shared input shape:

```json
{
  "action": "movie.lookup",
  "params": {
    "query": "Inception"
  }
}
```

Discovery comes from the same runtime catalog that powers the CLI:

- `lab help`
- `lab://catalog`
- `lab://<service>/actions`
- per-service `help`/`schema` actions

### HTTP API

The binary also mounts an Axum API:

- `GET /health`
- `GET /ready`
- `GET /v1/<service>/actions`
- per-service action routes under `/v1/<service>`

The API and MCP layers both dispatch through the same underlying service/action model.

### TUI

`lab plugins` launches the Ratatui-based plugin manager. The shipped implementation already:

- loads `.mcp.json` and `~/.lab/.env`
- runs background health checks
- toggles services on and off
- patches `.mcp.json` atomically with a sidecar lock + backups

## Supported service groups

`lab` organizes integrations by category through `PluginMeta`:

| Category | Services |
| --- | --- |
| Servarr | Radarr, Sonarr |
| Indexer | Prowlarr |
| Media | Plex, Tautulli, Overseerr |
| Download | SABnzbd, qBittorrent |
| Notes / Documents | Linkding, Memos, ByteStash, Paperless-ngx |
| Network / Infrastructure | Tailscale, UniFi, Unraid, Arcane |
| Notifications | Gotify, Apprise |
| AI / Inference | OpenAI, Qdrant, TEI |
| Bootstrap | Extract |

Implementation depth varies by service. The fastest way to check the current runtime surface is:

```bash
cargo run --all-features -- help --json
```

## Quick start

### 1. Build from source

The workspace requires **Rust 1.90** and uses edition **2024**.

```bash
git clone git@github.com:jmagar/lab.git
cd lab
cargo build --workspace --all-features
```

To install the binary locally from the workspace:

```bash
cargo install --path crates/lab --all-features
```

### 2. Add secrets and preferences

`lab` intentionally splits secrets from preferences:

- secrets: `~/.lab/.env`
- preferences: `~/.config/lab/config.toml`

Example `~/.lab/.env`:

```env
RADARR_URL=http://localhost:7878
RADARR_API_KEY=abc123

SABNZBD_URL=http://localhost:8080
SABNZBD_API_KEY=abc123

UNRAID_URL=https://tower.local/graphql
UNRAID_API_KEY=abc123

UNIFI_URL=https://unifi.local
UNIFI_API_KEY=abc123
```

Named multi-instance services follow the same pattern:

```env
UNRAID_NODE2_URL=https://tower-2.local/graphql
UNRAID_NODE2_API_KEY=abc123
```

Example `~/.config/lab/config.toml`:

```toml
[output]
format = "json"

[mcp]
transport = "stdio"
host = "127.0.0.1"
port = 8765
```

For repo-local development, the binary also loads a `.env` in the current working directory if present.

### 3. Inspect the catalog

```bash
lab help
lab help --json
```

### 4. Start the MCP server

```bash
lab serve
LAB_MCP_HTTP_TOKEN=... lab serve --transport http
lab serve --transport http --host 127.0.0.1 --port 8765
```

When the HTTP transport is running, all `/v1/*` API requests require a Bearer token matching `LAB_MCP_HTTP_TOKEN`:

```bash
curl -s -X POST http://127.0.0.1:8765/v1/radarr \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"system.status"}'
```

### 5. Use the operator commands

```bash
lab doctor
lab health
lab plugins
```

## Development commands

The repo ships a `Justfile` with the normal workspace tasks:

```bash
just check
just test
just lint
just deny
just build
just build-release
just fmt
```

Useful direct commands:

```bash
cargo test -p lab-apis
cargo test -p lab
cargo doc --no-deps --all-features
```

**Authoritative verification path** (exercises all feature-gated surfaces):

```bash
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo test --workspace --tests --no-fail-fast --all-features
```

## Design highlights

- **Two-crate split:** SDK logic stays reusable and transport-agnostic.
- **One tool per service:** MCP stays compact instead of exploding into hundreds of tiny tools.
- **Shared action catalog:** the same metadata drives CLI help, MCP discovery, and HTTP action listings.
- **Feature-gated services:** `lab-apis` owns the real feature flags; `lab` mirrors them.
- **Structured outputs:** success/error envelopes are designed for machine use, not just prose.
- **Safe local mutation:** the TUI `.mcp.json` patcher uses validation, locking, backups, and atomic writes.
- **Privacy posture:** no telemetry, analytics, or phone-home behavior beyond explicit service calls and explicit update operations.

## Docs map

Start with `docs/README.md`. The most useful topic docs are:

- `docs/ARCH.md` — crate split, runtime surfaces, shared contracts
- `docs/SERVICES.md` — service inventory and feature model
- `docs/CLI.md` — CLI behavior and operator commands
- `docs/MCP.md` — one-tool-per-service MCP design and envelopes
- `docs/CONFIG.md` — env/TOML ownership and multi-instance naming
- `docs/OPERATIONS.md` — CI, release, health, and repo workflows
- `docs/TESTING.md` — testing rules and verification expectations

## License

Workspace metadata declares `MIT OR Apache-2.0`.
