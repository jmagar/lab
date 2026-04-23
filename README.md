# lab

`lab` is a Rust workspace for running a homelab control plane from one codebase: a reusable SDK in `lab-apis`, plus a `lab` binary that exposes the same services through a CLI, an MCP server, an HTTP API, and a TUI plugin manager.

One binary. **23 services** (21 feature-gated + always-on `extract` and `marketplace`). Three runtime surfaces (CLI, MCP, HTTP API) that all dispatch through the same shared action catalog. **580 callable actions** across all services. One MCP tool per service, not hundreds.

## Current state

In the current `--all-features` build, the catalog registers **23 services** across 10 categories with **580 total callable actions**:

| Category | Services |
| --- | --- |
| Servarr | Radarr, Sonarr |
| Indexer | Prowlarr |
| Media | Plex, Tautulli, Overseerr |
| Download | SABnzbd, qBittorrent |
| Notes / Bookmarks | Linkding, Memos, ByteStash |
| Documents | Paperless-ngx |
| Network / Infrastructure | Tailscale, UniFi, Unraid, Arcane |
| Notifications | Gotify, Apprise |
| AI / Inference | OpenAI, Qdrant, TEI |
| Bootstrap | Extract, Marketplace (always-on) |

**Total callable actions: 580**

## Workspace layout

| Path | Role |
| --- | --- |
| `crates/lab-apis` | Pure Rust SDK: typed clients, request/response models, auth, shared HTTP behavior, health contracts, plugin metadata |
| `crates/lab` | Product binary: CLI, MCP server, HTTP API, TUI, config loading, output rendering, discovery catalog |
| `docs/` | Topic-based source-of-truth documentation |

The architectural rule: **shared service logic belongs in `lab-apis`; the `lab` crate adapts that logic for user-facing surfaces**.

---

## Quick start

### 1. Build from source

The workspace requires **Rust 1.90+** and uses edition **2024**.

```bash
git clone git@github.com:jmagar/lab.git
cd lab
cargo build --workspace --all-features
```

To install the binary locally:

```bash
cargo install --path crates/lab --all-features
```

### 2. Add secrets and preferences

`lab` splits secrets from preferences:

- **Secrets:** `~/.lab/.env`
- **Preferences:** `config.toml` (searched `./` → `~/.lab/` → `~/.config/lab/`)

Config loading order (highest priority wins):

1. Process environment variables
2. `~/.lab/.env` (via `dotenvy`)
3. `config.toml` (first found: `./config.toml` → `~/.lab/config.toml` → `~/.config/lab/config.toml`)
4. `.env` in the current working directory (non-fatal if missing)

Example `~/.lab/.env`:

```env
RADARR_URL=http://localhost:7878
RADARR_API_KEY=abc123

SONARR_URL=http://localhost:8989
SONARR_API_KEY=abc123

SABNZBD_URL=http://localhost:8080
SABNZBD_API_KEY=abc123

UNRAID_URL=https://tower.local/graphql
UNRAID_API_KEY=abc123

UNIFI_URL=https://unifi.local
UNIFI_API_KEY=abc123

PLEX_URL=http://localhost:32400
PLEX_TOKEN=abc123

GOTIFY_URL=http://localhost:80
GOTIFY_TOKEN=abc123
```

**Multi-instance services** follow the same pattern with a label:

```env
UNRAID_URL=https://tower.local/graphql
UNRAID_API_KEY=abc123

UNRAID_NODE2_URL=https://tower-2.local/graphql
UNRAID_NODE2_API_KEY=abc123
```

MCP callers select instances via `params.instance`; CLI via `--instance`.

Example `~/.lab/config.toml` (or `./config.toml` for per-repo overrides):

```toml
[output]
format = "json"

[log]
filter = "lab=debug,lab_apis=info"

[mcp]
transport = "http"
host = "0.0.0.0"
port = 9000

[api]
cors_origins = ["https://lab.example.com"]
```

See `config.example.toml` for all available settings with defaults.

### 3. Inspect the catalog

```bash
lab help
lab help --json
```

### 4. Start the MCP server

```bash
lab serve
LAB_MCP_HTTP_TOKEN=... lab serve
LAB_AUTH_MODE=oauth LAB_PUBLIC_URL=https://lab.example.com LAB_GOOGLE_CLIENT_ID=... LAB_GOOGLE_CLIENT_SECRET=... lab serve
lab serve --host 127.0.0.1 --port 8765
lab serve mcp --stdio
lab serve --services radarr,sonarr,plex
```

`lab serve` is the hosted runtime path. It always starts the HTTP server for:

- the product API
- the Labby web UI (when exported assets exist)
- OAuth metadata and token endpoints
- the hosted HTTP MCP surface at `/mcp`

When the exported Labby bundle exists at `apps/gateway-admin/out`, `lab serve`
also serves the web UI from the same origin. In that mode:

- Labby UI is available at `http://127.0.0.1:8765/`
- product API stays at `http://127.0.0.1:8765/v1/...`
- MCP over HTTP stays at `http://127.0.0.1:8765/mcp`

The separate Next dev server on `3000` is now a frontend development workflow only.

### 5. Use the operator commands

```bash
lab doctor       # Comprehensive health audit for all configured services
lab health       # Quick reachability check
lab plugins      # Launch the TUI plugin manager
lab oauth relay-local --machine dookie --port 38935
```

When a browser machine needs to catch a localhost OAuth redirect and forward it to a remote MCP
client, `lab oauth relay-local` can proxy the callback to a named target or an explicit Tailscale
URL without reimplementing the OAuth flow.

Minimal named-machine setup:

```toml
# ~/.lab/config.toml
[oauth.machines.dookie]
target_url = "http://100.88.16.79:38935/callback/dookie"
description = "dookie Codex callback listener"
default_port = 38935
```

```bash
# Run on the browser-side machine during login
lab oauth relay-local --machine dookie --port 38935
```

`relay-local` only forwards the final callback request. It does not exchange codes, mint tokens,
or replace the normal OAuth listener on the real MCP client machine.

---

## CLI

### Top-level commands

```
lab help              Print the service + action catalog
lab serve             Start the hosted lab server (API + web UI + OAuth + HTTP MCP)
lab serve mcp --stdio Start the MCP server over stdio for local clients
lab doctor            Audit configured services and report problems
lab health            Quick reachability check for configured services
lab plugins           Open the TUI plugin manager
lab extract           Scan appdata paths and extract service credentials
lab audit             Audit service onboarding against repo contract
lab scaffold          Generate a new service onboarding scaffold
lab install           Install services into .mcp.json
lab uninstall         Uninstall services from .mcp.json
lab init              First-time setup wizard
lab completions       Generate shell completions (bash, zsh, fish, etc.)
```

Global flag: `--json` — emit JSON instead of human-readable tables.

### `lab serve`

| Flag | Description |
| --- | --- |
| `--host <addr>` | HTTP bind host (default: `127.0.0.1`) |
| `--port <port>` | HTTP bind port (default: `8765`) |
| `--services <list>` | Comma-separated service filter (default: all) |

`lab serve` always starts the hosted HTTP server. That includes the API, OAuth endpoints, web UI when assets are available, and the HTTP MCP surface at `/mcp`.

Config precedence for the hosted listener: CLI args > env vars (`LAB_MCP_HTTP_HOST`, `LAB_MCP_HTTP_PORT`) > `config.toml` (first found from search order) > defaults.

The hosted HTTP server requires either `LAB_MCP_HTTP_TOKEN` in bearer mode or the OAuth settings (`LAB_AUTH_MODE=oauth`, `LAB_PUBLIC_URL`, and Google client credentials) in OAuth mode.

### `lab serve mcp --stdio`

Use this when you want a stdio MCP server for a local client such as Claude Desktop or an editor integration.

```bash
lab serve mcp --stdio
```

The hosted API/web UI server and the stdio MCP helper are now separate operator paths.

### `lab extract`

Scans a local or SSH appdata path and discovers service credentials.

```bash
lab extract /mnt/appdata                          # Scan and display
lab extract /mnt/appdata --apply                  # Write creds to ~/.lab/.env
lab extract /mnt/appdata --diff                   # Show changes vs current .env
lab extract user@host:/mnt/appdata --apply --yes  # SSH scan, auto-confirm
```

| Flag | Description |
| --- | --- |
| `--apply` | Write discovered creds to `~/.lab/.env` |
| `--diff` | Show what `--apply` would change (no writes) |
| `-y` / `--yes` | Skip destructive confirmation prompt |
| `--dry-run` | Print what would happen without writing |
| `--force` | Overwrite conflicting keys |
| `--json` | Render as JSON |
| `--env-path <path>` | Override target `.env` location |

The `.env` merge algorithm: backup before write, preserve comments and blanks, dedupe by key, skip-or-force conflicts, quote values with special chars, atomic write, idempotent check.

### Per-service CLI subcommands

Every compiled service has a CLI subcommand that maps to its action catalog:

```bash
lab radarr movie-lookup "Inception"
lab radarr system-status
lab sonarr series-list
lab sabnzbd queue-list
lab unifi devices-list
lab plex session-list
lab gotify message-send --title "Alert" --message "Test"
```

Destructive CLI commands require `-y`/`--yes` to run non-interactively. `--no-confirm` and `--dry-run` are also honored.

---

## MCP server

### One tool per service

`lab serve` starts the hosted lab server. That hosted runtime includes the API, OAuth endpoints,
and, when assets are available, the Labby web UI from the same port as the API and MCP endpoints.
Every service is exposed as **one MCP tool**
accepting a shared input shape:

```json
{
  "action": "movie.search",
  "params": { "query": "Inception" }
}
```

This keeps the total MCP tool count at ~23 instead of hundreds. The hosted MCP surface defaults to HTTP at `/mcp`. For stdio clients, use `lab serve mcp --stdio`.

### Tool list

When all features are enabled, the MCP server registers these tools:

| Tool | Description | Actions |
| --- | --- | ---: |
| `extract` | Scan appdata paths for service credentials | 5 |
| `marketplace` | Claude Code plugin marketplace manager | 9 |
| `radarr` | Radarr movie collection manager | 53 |
| `sonarr` | Sonarr TV series manager | 34 |
| `prowlarr` | Prowlarr indexer manager | 25 |
| `plex` | Plex media server | 29 |
| `tautulli` | Tautulli Plex analytics | 27 |
| `sabnzbd` | SABnzbd download client | 22 |
| `qbittorrent` | qBittorrent download client | 31 |
| `tailscale` | Tailscale VPN network | 24 |
| `linkding` | Linkding bookmark manager | 21 |
| `memos` | Memos note-taking service | 20 |
| `bytestash` | ByteStash snippet manager | 18 |
| `paperless` | Paperless-ngx document manager | 31 |
| `arcane` | Arcane Docker management UI | 23 |
| `unraid` | Unraid server management | 30 |
| `unifi` | UniFi network management | 81 |
| `overseerr` | Overseerr media request manager | 30 |
| `gotify` | Gotify push notifications | 26 |
| `openai` | OpenAI API client | 6 |
| `qdrant` | Qdrant vector database | 15 |
| `tei` | HF Text Embeddings Inference | 10 |
| `apprise` | Apprise notification dispatcher | 10 |
| | **Total** | **580** |

Additionally, `lab_admin` (onboarding audit, 3 actions) is available as a runtime opt-in tool when `LAB_ADMIN_ENABLED=1`.

### Built-in actions

Every tool automatically supports two built-in actions:

- `help` — list available actions for this service
- `schema` — return the parameter schema for a named action

```json
radarr({ "action": "help" })
radarr({ "action": "schema", "params": { "action": "movie.add" } })
```

### MCP resources

Two read-only resources are exposed:

- `lab://catalog` — full discovery document for all services
- `lab://<service>/actions` — per-service action list

### Elicitation (destructive action confirmation)

When an MCP client supports elicitation, destructive actions trigger an interactive confirmation prompt before executing. The server sends a form with a `confirm: boolean` field that the client must accept.

For clients that do **not** support elicitation (headless, CI, automated agents), destructive actions can be confirmed by passing `"confirm": true` in the `params` object:

```json
radarr({ "action": "movie.delete", "params": { "id": 123, "confirm": true } })
```

Without confirmation, destructive actions return a `confirmation_required` error.

### Structured error envelopes

Every MCP tool failure returns a JSON envelope with a stable `kind` tag:

```json
{ "kind": "unknown_action", "message": "...", "valid": ["movie.search", ...], "hint": "movie.serch" }
{ "kind": "missing_param", "message": "...", "param": "query" }
{ "kind": "unknown_instance", "message": "...", "valid": ["default", "node2"] }
{ "kind": "rate_limited", "message": "...", "retry_after_ms": 5000 }
{ "kind": "confirmation_required", "message": "..." }
```

---

## HTTP API

When `lab serve --transport http` is running, an axum HTTP API is mounted alongside the MCP server.
If exported Labby assets are available, the same server also hosts the web UI at `/`.

### Global endpoints (no auth required)

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/` | Labby web UI shell (when exported assets are available) |
| `GET` | `/gateways/`, `/gateway/`, `/docs/`, etc. | Labby SPA routes (when exported assets are available) |
| `GET` | `/health` | Liveness probe |
| `GET` | `/ready` | Readiness probe |

### Protected endpoints (Bearer auth required)

All `/v1/*` routes require `Authorization: Bearer $LAB_MCP_HTTP_TOKEN`.

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/v1/{service}/actions` | List actions for a service |
| `POST` | `/v1/{service}` | Dispatch an action |
| `GET` | `/v1/openapi.json` | OpenAPI 3.1 spec |
| `GET` | `/v1/docs` | Scalar API documentation UI |

### Action dispatch

The HTTP API mirrors the MCP action+params dispatch shape:

```bash
curl -s -X POST http://127.0.0.1:8765/v1/radarr \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"movie.search","params":{"query":"Inception"}}'
```

### Per-service routes

Every compiled service gets a route group under `/v1/<service>`:

| Service | Route | Feature flag |
| --- | --- | --- |
| extract | `/v1/extract` | always-on |
| marketplace | `/v1/marketplace` | always-on |
| radarr | `/v1/radarr` | `radarr` |
| sonarr | `/v1/sonarr` | `sonarr` |
| prowlarr | `/v1/prowlarr` | `prowlarr` |
| plex | `/v1/plex` | `plex` |
| tautulli | `/v1/tautulli` | `tautulli` |
| sabnzbd | `/v1/sabnzbd` | `sabnzbd` |
| qbittorrent | `/v1/qbittorrent` | `qbittorrent` |
| tailscale | `/v1/tailscale` | `tailscale` |
| linkding | `/v1/linkding` | `linkding` |
| memos | `/v1/memos` | `memos` |
| bytestash | `/v1/bytestash` | `bytestash` |
| paperless | `/v1/paperless` | `paperless` |
| arcane | `/v1/arcane` | `arcane` |
| unraid | `/v1/unraid` | `unraid` |
| unifi | `/v1/unifi` | `unifi` |
| overseerr | `/v1/overseerr` | `overseerr` |
| gotify | `/v1/gotify` | `gotify` |
| openai | `/v1/openai` | `openai` |
| qdrant | `/v1/qdrant` | `qdrant` |
| tei | `/v1/tei` | `tei` |
| apprise | `/v1/apprise` | `apprise` |

Services filtered out via `lab serve --services <list>` will not have their routes mounted, even if compiled in.

### Destructive action confirmation (HTTP)

For the HTTP API, destructive actions require `"confirm": true` in the JSON request `params` object:

```bash
curl -s -X POST http://127.0.0.1:8765/v1/radarr \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"movie.delete","params":{"id":123,"confirm":true}}'
```

Without confirmation, the API returns `400` with `kind: "confirmation_required"`.

### Status code mapping

| Error `kind` | HTTP Status |
| --- | --- |
| `auth_failed` | 401 |
| `not_found` | 404 |
| `rate_limited` | 429 |
| `validation_failed`, `missing_param`, `invalid_param` | 422 |
| `unknown_action`, `unknown_instance` | 400 |
| `confirmation_required` | 400 |
| `network_error`, `server_error` | 502 |
| `decode_error`, `internal_error` | 500 |

### Middleware stack

1. **SetRequestId** — generates UUID v4, propagated as `x-request-id`
2. **TraceLayer** — structured tracing spans per request
3. **PropagateRequestId** — echoes `x-request-id` back in response
4. **TimeoutLayer** — 30s default
5. **CompressionLayer** — gzip
6. **CorsLayer** — explicit allowlist (loopback origins always allowed; additional via `LAB_CORS_ORIGINS`)

### OpenAPI documentation

When running in HTTP mode, the API serves:

- `GET /v1/openapi.json` — OpenAPI 3.1 specification
- `GET /v1/docs` — Scalar interactive API documentation

Both require bearer authentication.

---

## TUI plugin manager

`lab plugins` launches a Ratatui-based plugin manager with two tabs:

**Services tab:**
- Browse all compiled-in services with category grouping
- Live health check indicators (reachable, auth OK, latency)
- Configure env vars per service
- Toggle services on/off in `.mcp.json`
- Atomic `.mcp.json` patching with validation, locking, and backups

**Plugins tab:**
- Browse available marketplaces (Claude Code, Codex, Gemini CLI extensions)

Health checks run concurrently in the background (max 5 in parallel) and display per-service reachability, authentication status, and latency.

---

## Service action catalogs

### Extract (always-on)

Bootstrap utility — scans appdata paths and discovers service credentials.

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `scan` | Scan an appdata path and return discovered credentials | |
| `apply` | Scan and write credentials into `~/.lab/.env` | **yes** |
| `diff` | Show what `apply` would change (no writes) | |

### Marketplace (always-on)

Claude Code plugin marketplace manager. Reads `~/.claude/plugins/` and wraps the destructive `claude plugin …` commands. See `docs/MARKETPLACE.md` for details.

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `sources.list` | List configured marketplaces | |
| `sources.add` | Register a new marketplace via `claude plugin marketplace add` | **yes** |
| `plugins.list` | List all plugins across marketplaces with installed state | |
| `plugin.get` | Get a single plugin by `name@marketplace` id | |
| `plugin.artifacts` | List artifact files shipped with an installed plugin | |
| `plugin.install` | Install a plugin via `claude plugin install` | **yes** |
| `plugin.uninstall` | Uninstall a plugin via `claude plugin uninstall` | **yes** |

### Radarr (Servarr — Movie collection manager)

Env: `RADARR_URL`, `RADARR_API_KEY` · Default port: `7878`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `system.status` | Return system status and version | |
| `system.health` | Return health check results | |
| `system.disk-space` | Return disk space for all drives | |
| `system.logs` | Return available log files | |
| `system.updates` | Return available updates | |
| `system.restart` | Restart the application | **yes** |
| `system.backup` | List backup files | |
| `system.task` | List scheduled background tasks | |
| `system.task-execute` | Execute a scheduled task immediately | **yes** |
| `movie.list` | List all movies | |
| `movie.get` | Get a movie by ID | |
| `movie.lookup` | Search TMDB/IMDB for movies | |
| `movie.add` | Add a movie for monitoring | |
| `movie.edit` | Update an existing movie | |
| `movie.delete` | Delete a movie | **yes** |
| `queue.list` | List download queue | |
| `queue.remove` | Remove a queue item | **yes** |
| `calendar.list` | List upcoming releases | |
| `command.refresh` | Refresh movie metadata | |
| `command.search` | Trigger a file search | |
| `command.get` | Get command status | |
| `history.list` | List download history | |
| `history.movie` | History for a specific movie | |
| `history.failed-retry` | Retry a failed download | |
| `blocklist.list` | List blocked releases | |
| `blocklist.delete` | Delete a blocklist entry | **yes** |
| `release.search` | Search indexers for releases | |
| `release.grab` | Grab a release for download | |
| `indexer.list` | List configured indexers | |
| `indexer.test` | Test an indexer connection | |
| `quality-profile.list` | List quality profiles | |
| `quality-definition.list` | List quality definitions | |
| `root-folder.list` | List root folders | |
| `tag.list` | List all tags | |
| `tag.detail-list` | List tags with details | |
| `download-client.list` | List download clients | |
| `download-client.test` | Test a download client | |
| `remote-path-mapping.list` | List remote path mappings | |
| `config.host` | Get host configuration | |
| `config.naming` | Get file naming configuration | |
| `config.ui` | Get UI configuration | |
| `notification.list` | List configured notifications | |
| `notification.test` | Test a notification | |
| `import-list.list` | List import lists | |
| `import-list.exclusion-list` | List import list exclusions | |
| `language.list` | List available languages | |
| `metadata.list` | List metadata providers | |
| `filesystem.list` | Browse server filesystem | |
| `wanted.missing` | List missing movies | |
| `wanted.cutoff` | List movies below quality cutoff | |
| `customformat.list` | List custom formats | |

### Sonarr (Servarr — TV series manager)

Env: `SONARR_URL`, `SONARR_API_KEY` · Default port: `8989`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `series.list` | List all TV series | |
| `series.get` | Get a series by ID | |
| `series.lookup` | Search for series (TVDB) | |
| `series.add` | Add a series for monitoring | |
| `series.edit` | Update an existing series | |
| `series.delete` | Delete a series | **yes** |
| `episode.list` | List episodes for a series | |
| `episode.get` | Get an episode by ID | |
| `episode.monitor` | Set monitored state for episodes | |
| `queue.list` | List download queue | |
| `queue.delete` | Remove a queue item | **yes** |
| `history.list` | List download history | |
| `history.series` | History for a specific series | |
| `history.failed-retry` | Retry a failed download | |
| `wanted.list` | List missing episodes | |
| `wanted.cutoff` | List episodes below quality cutoff | |
| `calendar.list` | List upcoming air dates | |
| `health` | Return health check results | |
| `system.status` | Return system status | |
| `system.restart` | Restart the application | **yes** |
| `system.backup` | List backup files | |
| `tag.list` | List all tags | |
| `tag.create` | Create a tag | |
| `tag.delete` | Delete a tag | **yes** |
| `rootfolder.list` | List root folders | |
| `qualityprofile.list` | List quality profiles | |
| `languageprofile.list` | List language profiles | |
| `release.search` | Search for releases | |
| `release.grab` | Grab a release | |
| `blocklist.list` | List blocklisted releases | |
| `blocklist.delete` | Remove from blocklist | **yes** |
| `episodefile.delete` | Delete an episode file from disk | **yes** |

### Prowlarr (Indexer manager)

Env: `PROWLARR_URL`, `PROWLARR_API_KEY` · Default port: `9696`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `indexer.list` | List all indexers | |
| `indexer.get` | Get an indexer by ID | |
| `indexer.add` | Create a new indexer | |
| `indexer.edit` | Update an indexer | |
| `indexer.delete` | Delete an indexer | **yes** |
| `indexer.test` | Test an indexer | |
| `indexer.testall` | Test all indexers | |
| `indexer.categories` | List indexer categories | |
| `indexer.stats` | Get indexer statistics | |
| `indexer.status` | Get indexers with errors | |
| `indexer.search` | Search across indexers | |
| `indexer.grab` | Grab a release | |
| `history.list` | Get history with filters | |
| `history.indexer` | History for a specific indexer | |
| `application.list` | List connected applications | |
| `application.get` | Get an application by ID | |
| `application.add` | Create an application | |
| `application.delete` | Delete an application | **yes** |
| `system.status` | Get system status | |
| `system.health` | Get health alerts | |
| `system.restart` | Restart the server | **yes** |
| `system.backup` | List backup files | |
| `tag.list` | List all tags | |

### Plex (Media server)

Env: `PLEX_URL`, `PLEX_TOKEN` · Default port: `32400`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `health` | Check server reachability and auth | |
| `server.info` | Get server identity and configuration | |
| `server.capabilities` | Get media provider capabilities | |
| `library.list` | List all library sections | |
| `library.get` | Get metadata for a library | |
| `library.scan` | Trigger a library scan | |
| `library.refresh` | Force metadata refresh for a library | **yes** |
| `library.browse` | List content in a library | |
| `library.empty-trash` | Permanently remove deleted items | **yes** |
| `media.search` | Search across libraries | |
| `media.get` | Get metadata for a media item | |
| `session.list` | List active playback sessions | |
| `session.terminate` | Terminate a playback session | **yes** |
| `session.history` | Get recently played items | |
| `playlist.list` | List playlists | |
| `playlist.get` | Get a playlist | |
| `playlist.create` | Create a playlist | **yes** |
| `playlist.delete` | Delete a playlist | **yes** |
| `metadata.delete` | Delete a metadata item | **yes** |
| `metadata.edit` | Edit metadata fields | |
| `metadata.refresh` | Re-download metadata from agents | |
| `hubs.continue-watching` | Get Continue Watching hub | |
| `butler.list` | List butler tasks | |
| `butler.run` | Trigger a butler task | |
| `item.scrobble` | Mark as played | |
| `item.unscrobble` | Mark as unplayed | |
| `updater.status` | Get update status | |

### Tautulli (Plex analytics)

Env: `TAUTULLI_URL`, `TAUTULLI_API_KEY` · Default port: `8181`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema for an action | |
| `activity.list` | Get current Plex activity | |
| `activity.stream` | Get details for an active session | |
| `history.list` | Get play history with filters | |
| `users.list` | List all users | |
| `users.get` | Get user details | |
| `users.watch_time` | Get watch time statistics | |
| `users.player_stats` | Get player statistics | |
| `libraries.list` | List tracked libraries | |
| `libraries.get` | Get library details | |
| `libraries.media_info` | Get media info for a library | |
| `stats.home` | Get home stats | |
| `stats.plays_by_date` | Play counts by date | |
| `media.recently-added` | Recently added media | |
| `media.metadata` | Get media metadata | |
| `media.children` | Get children metadata | |
| `media.export-metadata` | Export metadata | |
| `user.item-stats` | User statistics for a media item | |
| `user.delete-history` | Delete all play history for a user | **yes** |
| `plays.by-day` | Play count by day of week | |
| `plays.by-hour` | Play count by hour of day | |
| `plays.by-stream-type` | Play count by stream type | |
| `plays.by-month` | Play count by month | |
| `server.pms-update` | Check for Plex updates | |
| `system.info` | Get Tautulli server info | |
| `system.settings` | Get Tautulli settings | |

### SABnzbd (Usenet download client)

Env: `SABNZBD_URL`, `SABNZBD_API_KEY` · Default port: `8080`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `server.version` | Return version string | |
| `server.stats` | Download statistics | |
| `server.warnings` | List warnings | |
| `server.fullstatus` | Detailed server status | |
| `queue.list` | List download queue | |
| `queue.delete` | Delete a queue item | **yes** |
| `queue.pause` | Pause the queue | |
| `queue.resume` | Resume the queue | |
| `queue.speed.limit` | Set speed limit (KB/s) | |
| `queue.addurl` | Add a URL to the queue | |
| `queue.set-complete-action` | Set queue completion action | |
| `history.list` | List download history | |
| `history.delete` | Delete a history item | **yes** |
| `history.purge` | Purge all history | **yes** |
| `history.retry` | Retry a failed item | |
| `history.retry-all` | Retry all failed items | |
| `pp.pause` | Pause post-processing | |
| `pp.resume` | Resume post-processing | |
| `rss.fetch-now` | Trigger RSS feed fetch | |
| `config.get` | Get configuration (redacted) | |
| `category.list` | List download categories | |

### qBittorrent (BitTorrent download client)

Env: `QBITTORRENT_URL`, `QBITTORRENT_PASSWORD` (optional: `QBITTORRENT_USERNAME`, `QBITTORRENT_SID`) · Default port: `8080`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `app.version` | Get application version | |
| `app.preferences` | Get application preferences | |
| `transfer.info` | Global transfer info | |
| `transfer.download.limit` | Set global download limit | |
| `transfer.upload.limit` | Set global upload limit | |
| `transfer.toggle-speed-limits` | Toggle alternative speed limits | |
| `torrent.list` | List torrents | |
| `torrent.properties` | Get torrent properties | |
| `torrent.trackers` | Get tracker list | |
| `torrent.files` | Get file list | |
| `torrent.pause` | Pause torrents | |
| `torrent.resume` | Resume torrents | |
| `torrent.delete` | Delete torrents | **yes** |
| `torrent.recheck` | Force re-hash verification | |
| `torrent.reannounce` | Force re-announce | |
| `torrent.add` | Add torrents by URL | |
| `torrent.category.set` | Set torrent category | |
| `torrent.download.limit` | Set per-torrent download limit | |
| `torrent.upload.limit` | Set per-torrent upload limit | |
| `torrent.set-file-prio` | Set file download priority | |
| `torrent.set-location` | Move torrents to new location | |
| `torrent.add-tags` | Add tags to torrents | |
| `torrent.remove-tags` | Remove tags from torrents | |
| `torrent.set-share-limits` | Set share limits | |
| `category.list` | List categories | |
| `category.create` | Create a category | |
| `category.edit` | Edit a category | |
| `log.list` | Get application log | |
| `sync.maindata` | Get sync maindata (full or delta) | |

### Tailscale (VPN network)

Env: `TAILSCALE_API_KEY` (optional: `TAILSCALE_TAILNET`, `TAILSCALE_BASE_URL`)

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `device.list` | List all devices | |
| `device.get` | Get device details | |
| `device.delete` | Remove a device | **yes** |
| `device.authorize` | Authorize/de-authorize a device | |
| `device.routes-get` | Get advertised/accepted routes | |
| `device.routes-set` | Set subnet routes | |
| `device.tag` | Set tags on a device | |
| `device.expire` | Expire device key | **yes** |
| `key.list` | List auth keys | |
| `key.get` | Get auth key details | |
| `key.create` | Create an auth key | |
| `key.delete` | Delete an auth key | **yes** |
| `dns.nameservers` | Get DNS nameservers | |
| `dns.search_paths` | Get DNS search paths | |
| `dns.split-get` | Get split DNS config | |
| `dns.split-set` | Replace split DNS config | |
| `acl.get` | Get ACL policy | |
| `acl.validate` | Validate ACL policy | |
| `acl.set` | Set ACL policy | |
| `user.list` | List all users | |
| `tailnet.settings-get` | Get tailnet settings | |
| `tailnet.settings-patch` | Patch tailnet settings | |

### Linkding (Bookmark manager)

Env: `LINKDING_URL`, `LINKDING_TOKEN` · Default port: `9090`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `bookmark.list` | List bookmarks with search/pagination | |
| `bookmark.archived.list` | List archived bookmarks | |
| `bookmark.get` | Get a bookmark by ID | |
| `bookmark.check` | Check if a URL is bookmarked | |
| `bookmark.create` | Create a bookmark | |
| `bookmark.update` | Partially update a bookmark | |
| `bookmark.archive` | Archive a bookmark | |
| `bookmark.unarchive` | Unarchive a bookmark | |
| `bookmark.delete` | Delete a bookmark | **yes** |
| `bookmark.assets` | List bookmark assets | |
| `bookmark.assets-upload` | Upload an asset file | |
| `tag.list` | List all tags | |
| `tag.get` | Get a tag by ID | |
| `tag.create` | Create a tag | |
| `bundle.list` | List bundles (saved searches) | |
| `bundle.create` | Create a bundle | |
| `bundle.update` | Update a bundle | |
| `bundle.delete` | Delete a bundle | **yes** |
| `user.profile` | Get user profile | |

### Memos (Note-taking service)

Env: `MEMOS_URL`, `MEMOS_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `memos.list` | List memos | |
| `memos.get` | Get a memo | |
| `memos.create` | Create a memo | |
| `memos.update` | Update a memo | |
| `memos.delete` | Delete a memo | **yes** |
| `tags.list` | List tags | |
| `workspace.profile` | Get workspace profile | |
| `user.me` | Get current user | |
| `user.list` | List users | |
| `user.stats` | Get user statistics | |
| `webhook.list` | List webhooks | |
| `webhook.create` | Create a webhook | |
| `attachment.upload` | Upload an attachment | |
| `attachment.delete` | Delete an attachment | **yes** |
| `memo.comment-list` | List memo comments | |
| `memo.comment-create` | Create a comment | |
| `memo.share-list` | List memo shares | |
| `memo.share-create` | Create a share | |

### ByteStash (Snippet manager)

Env: `BYTESTASH_URL`, `BYTESTASH_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `auth.config` | Get auth configuration | |
| `auth.register` | Register a new account | **yes** |
| `auth.login` | Login and get a token | |
| `snippets.list` | List snippets | |
| `snippets.get` | Get a snippet | |
| `snippets.create` | Create a snippet | **yes** |
| `snippets.update` | Update a snippet | **yes** |
| `snippets.delete` | Delete a snippet | **yes** |
| `snippets.public.list` | List public snippets | |
| `snippets.public.get` | Get a public snippet | |
| `snippets.share.create` | Create a share link | **yes** |
| `snippets.share.get` | Get share details | |
| `categories.list` | List categories | |
| `users.list` | List users | |
| `users.toggle-active` | Toggle user active state | **yes** |
| `users.delete` | Delete a user | **yes** |

### Paperless-ngx (Document manager)

Env: `PAPERLESS_URL`, `PAPERLESS_TOKEN` · Default port: `8000`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `documents.list` | List documents with filters | |
| `documents.get` | Get a document by ID | |
| `documents.metadata` | Get full document metadata | |
| `documents.update` | Update a document | **yes** |
| `documents.delete` | Delete a document | **yes** |
| `document.upload` | Upload a document | |
| `document.download` | Download a document (base64) | |
| `document.bulk-edit` | Bulk edit documents | **yes** |
| `tags.list` | List tags | |
| `tags.get` | Get a tag | |
| `tags.create` | Create a tag | **yes** |
| `tags.delete` | Delete a tag | **yes** |
| `tag.update` | Update a tag | |
| `correspondents.list` | List correspondents | |
| `correspondents.get` | Get a correspondent | |
| `correspondents.create` | Create a correspondent | **yes** |
| `correspondents.delete` | Delete a correspondent | **yes** |
| `document_types.list` | List document types | |
| `document_types.get` | Get a document type | |
| `document_types.create` | Create a document type | **yes** |
| `document_types.delete` | Delete a document type | **yes** |
| `statistics` | Get instance statistics | |
| `tasks.list` | List async task status | |
| `saved-view.list` | List saved views | |
| `saved-view.create` | Create a saved view | |
| `custom-field.list` | List custom fields | |
| `custom-field.create` | Create a custom field | |
| `storage-path.list` | List storage paths | |
| `storage-path.create` | Create a storage path | |

### Arcane (Docker management UI)

Env: `ARCANE_URL`, `ARCANE_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `health` | Check server health | |
| `environment.list` | List environments | |
| `environment.get` | Get an environment | |
| `container.list` | List containers | |
| `container.get` | Get a container | |
| `container.start` | Start a container | |
| `container.stop` | Stop a container | |
| `container.restart` | Restart a container | |
| `container.redeploy` | Redeploy a container | **yes** |
| `project.list` | List Compose projects | |
| `project.create` | Create a project | |
| `project.up` | Start a project | |
| `project.down` | Stop and remove a project | **yes** |
| `project.redeploy` | Redeploy a project | |
| `volume.list` | List volumes | |
| `volume.delete` | Delete a volume | **yes** |
| `volume.prune` | Prune unused volumes | **yes** |
| `image.list` | List images | |
| `image.pull` | Pull an image | |
| `image.prune` | Prune unused images | **yes** |
| `image.update-summary` | Check for image updates | |

### Unraid (Server management)

Env: `UNRAID_URL`, `UNRAID_API_KEY` · Default port: `31337` · Supports multi-instance

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `system.info` | Get system information | |
| `system.metrics` | Get system metrics | |
| `system.array` | Get array status | |
| `system.online` | Check server connectivity | |
| `docker.list` | List Docker containers | |
| `docker.start` | Start a container | **yes** |
| `docker.stop` | Stop a container | **yes** |
| `docker.restart` | Restart a container | **yes** |
| `disk.list` | List disks | |
| `vm.list` | List VMs | |
| `vm.start` | Start a VM | **yes** |
| `vm.stop` | Stop a VM | **yes** |
| `vm.pause` | Pause a VM | **yes** |
| `vm.resume` | Resume a VM | |
| `notification.list` | List notifications | |
| `notification.create` | Create a notification | |
| `notification.archive` | Archive a notification | **yes** |
| `parity.history` | Get parity check history | |
| `parity.check-start` | Start a parity check | |
| `parity.check-pause` | Pause a parity check | |
| `parity.check-cancel` | Cancel a parity check | **yes** |
| `share.list` | List shares | |
| `plugin.list` | List plugins | |
| `network.list` | List networks | |
| `ups.devices` | List UPS devices | |
| `ups.config` | Get UPS configuration | |
| `log.read` | Read system log | |
| `flash.status` | Get flash drive status | |
| `flash.backup` | Backup flash drive | **yes** |

### UniFi (Network management)

Env: `UNIFI_URL`, `UNIFI_API_KEY` · Default port: `443`

The deepest service in the catalog with 85 actions across 11 domains.

<details>
<summary>Full action list (85 actions)</summary>

**System & Sites**

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `system.info` | Get application version and runtime | |
| `sites.list` | List local sites | |
| `wans.list` | List WAN interfaces | |
| `wan.get` | Inspect a WAN interface | |
| `vpn.site-to-site-tunnels.list` | List VPN tunnels | |
| `vpn.servers.list` | List VPN servers | |
| `radius.profiles.list` | List RADIUS profiles | |
| `device-tags.list` | List device tags | |
| `dpi.categories.list` | List DPI categories | |
| `dpi.applications.list` | List DPI applications | |
| `countries.list` | List supported countries | |

**Devices**

| Action | Description | Destructive |
| --- | --- | --- |
| `devices.list` | List adopted devices | |
| `devices.get` | Inspect one device | |
| `devices.stats` | Get device statistics | |
| `pending-devices.list` | List pending devices | |
| `devices.create` | Adopt a device | **yes** |
| `devices.port-action` | Perform port action | **yes** |
| `devices.action` | Perform device action (restart, etc.) | **yes** |
| `devices.delete` | Remove a device | **yes** |
| `device.update` | Update device configuration | **yes** |

**Clients**

| Action | Description | Destructive |
| --- | --- | --- |
| `clients.list` | List active clients | |
| `clients.get` | Inspect one client | |
| `clients.action` | Perform client action | **yes** |
| `client.history` | Get client connection history | |
| `client.block` | Block a client | **yes** |
| `client.unblock` | Unblock a client | |

**Networks**

| Action | Description | Destructive |
| --- | --- | --- |
| `networks.list` | List networks | |
| `networks.get` | Inspect one network | |
| `networks.references` | List network references | |
| `networks.create` | Create a network | **yes** |
| `networks.update` | Update a network | **yes** |
| `networks.delete` | Delete a network | **yes** |

**Wi-Fi**

| Action | Description | Destructive |
| --- | --- | --- |
| `wifi.broadcasts.list` | List SSIDs | |
| `wifi.broadcasts.get` | Inspect one SSID | |
| `wifi.broadcasts.create` | Create an SSID | **yes** |
| `wifi.broadcasts.update` | Update an SSID | **yes** |
| `wifi.broadcasts.delete` | Delete an SSID | **yes** |
| `wifi.update` | Update Wi-Fi configuration | **yes** |

**Hotspot**

| Action | Description | Destructive |
| --- | --- | --- |
| `hotspot.vouchers.list` | List vouchers | |
| `hotspot.vouchers.get` | Inspect one voucher | |
| `hotspot.vouchers.create` | Create a voucher | **yes** |
| `hotspot.vouchers.delete` | Delete vouchers | **yes** |

**Firewall**

| Action | Description | Destructive |
| --- | --- | --- |
| `firewall.zones.list` | List firewall zones | |
| `firewall.zones.get` | Inspect one zone | |
| `firewall.zones.create` | Create a zone | **yes** |
| `firewall.zones.update` | Update a zone | **yes** |
| `firewall.zones.delete` | Delete a zone | **yes** |
| `firewall.policies.list` | List policies | |
| `firewall.policies.get` | Inspect one policy | |
| `firewall.policies.create` | Create a policy | **yes** |
| `firewall.policies.update` | Replace a policy | **yes** |
| `firewall.policies.patch` | Patch a policy | **yes** |
| `firewall.policies.ordering.get` | Get policy ordering | |
| `firewall.policies.ordering.set` | Set policy ordering | **yes** |

**ACL**

| Action | Description | Destructive |
| --- | --- | --- |
| `acl.rules.list` | List ACL rules | |
| `acl.rules.get` | Inspect one rule | |
| `acl.rules.create` | Create a rule | **yes** |
| `acl.rules.update` | Update a rule | **yes** |
| `acl.rules.delete` | Delete a rule | **yes** |
| `acl.rules.ordering.get` | Get rule ordering | |
| `acl.rules.ordering.set` | Set rule ordering | **yes** |

**Switching**

| Action | Description | Destructive |
| --- | --- | --- |
| `switching.switch-stacks.list` | List switch stacks | |
| `switching.switch-stacks.get` | Inspect one stack | |
| `switching.mc-lag-domains.list` | List MC-LAG domains | |
| `switching.mc-lag-domains.get` | Inspect one MC-LAG domain | |
| `switching.lags.list` | List LAGs | |
| `switching.lags.get` | Inspect one LAG | |
| `port-profile.list` | List port profiles | |
| `port-profile.create` | Create a port profile | **yes** |
| `port-profile.update` | Update a port profile | **yes** |

**DNS**

| Action | Description | Destructive |
| --- | --- | --- |
| `dns.policies.list` | List DNS policies | |
| `dns.policies.get` | Inspect one policy | |
| `dns.policies.create` | Create a policy | **yes** |
| `dns.policies.update` | Update a policy | **yes** |
| `dns.policies.delete` | Delete a policy | **yes** |

**Traffic**

| Action | Description | Destructive |
| --- | --- | --- |
| `traffic-matching-lists.list` | List traffic matching lists | |
| `traffic-matching-lists.get` | Inspect one list | |
| `traffic-matching-lists.create` | Create a list | **yes** |
| `traffic-matching-lists.update` | Update a list | **yes** |
| `traffic-matching-lists.delete` | Delete a list | **yes** |

</details>

### Overseerr (Media request manager)

Env: `OVERSEERR_URL`, `OVERSEERR_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `health` | Check server health | |
| `status` | Get server status | |
| `request.list` | List media requests | |
| `request.get` | Get a request | |
| `request.create` | Create a request | |
| `request.approve` | Approve a request | |
| `request.decline` | Decline a request | |
| `request.delete` | Delete a request | **yes** |
| `request.retry` | Retry a request | |
| `request.count` | Get request count | |
| `movie.search` | Search for movies | |
| `movie.get` | Get movie details | |
| `tv.search` | Search for TV shows | |
| `tv.get` | Get TV show details | |
| `user.list` | List users | |
| `user.get` | Get a user | |
| `user.requests` | Get user's requests | |
| `user.quota` | Get user's quota | |
| `user.edit` | Edit user settings | |
| `issue.list` | List issues | |
| `issue.get` | Get an issue | |
| `issue.create` | Create an issue | |
| `issue.comment` | Comment on an issue | |
| `issue.update` | Update an issue | |
| `media.delete` | Delete a media item | **yes** |
| `media.update-status` | Update media status | |
| `job.run` | Run a scheduled job | |
| `discover.trending` | Get trending media | |

### Gotify (Push notifications)

Env: `GOTIFY_URL` (optional: `GOTIFY_TOKEN`, `GOTIFY_APP_TOKEN`, `GOTIFY_CLIENT_TOKEN`) · Default port: `80`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `server.health` | Check server health | |
| `server.version` | Get server version | |
| `message.send` | Send a push message | |
| `message.list` | List messages | |
| `message.delete` | Delete a message | **yes** |
| `message.purge` | Purge all messages | **yes** |
| `app.list` | List applications | |
| `app.create` | Create an application | |
| `app.delete` | Delete an application | **yes** |
| `application.update` | Update an application | |
| `application.messages` | List messages for an app | |
| `application.messages-delete` | Delete messages for an app | **yes** |
| `client.list` | List clients | |
| `client.create` | Create a client | |
| `client.delete` | Delete a client | **yes** |
| `client.update` | Update a client | |
| `plugin.list` | List plugins | |
| `plugin.enable` | Enable a plugin | |
| `plugin.disable` | Disable a plugin | |
| `plugin.config-get` | Get plugin configuration | |
| `plugin.config-set` | Set plugin configuration | |
| `user.list` | List users | |
| `user.create` | Create a user | |
| `user.delete` | Delete a user | **yes** |

### OpenAI (API client)

Env: `OPENAI_API_KEY` (optional: `OPENAI_URL`, `OPENAI_ORG_ID`)

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `model.list` | List available models | |
| `chat.complete` | Create a chat completion | |
| `embed.create` | Create embeddings | |
| `server.health` | Check API reachability | |

### Qdrant (Vector database)

Env: `QDRANT_URL` (optional: `QDRANT_API_KEY`) · Default port: `6333`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `server.health` | Check server health | |
| `collections.list` | List collections | |
| `collections.get` | Get collection metadata | |
| `collection.create` | Create a collection | |
| `collection.delete` | Delete a collection | **yes** |
| `point.upsert` | Upsert points | |
| `point.search` | Search nearest neighbours | |
| `point.query` | Universal query endpoint | |
| `point.scroll` | Scroll through points | |
| `point.count` | Count points | |
| `point.delete` | Delete points | **yes** |
| `snapshot.create` | Create a snapshot | |
| `index.create` | Create a payload index | |

### TEI (Text Embeddings Inference)

Env: `TEI_URL`, `TEI_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `server.health` | Check server health | |
| `server.info` | Get model and runtime metadata | |
| `embed.create` | Generate embeddings | |
| `embed.rerank` | Rerank texts against a query | |
| `embed.tokenize` | Tokenize inputs | |
| `embed.similarity` | Compute similarity scores | |
| `embed.sparse` | Generate sparse embeddings | |
| `embed.openai` | OpenAI-compatible embeddings | |

### Apprise (Notification dispatcher)

Env: `APPRISE_URL`, `APPRISE_API_KEY`

| Action | Description | Destructive |
| --- | --- | --- |
| `help` | Show action catalog | |
| `schema` | Return parameter schema | |
| `server.health` | Check server health | |
| `server.details` | Get server details and loaded plugins | |
| `notify.send` | Send a stateless notification | |
| `notify.key.send` | Send using a stored config key | |
| `config.add` | Store a config blob under a key | |
| `config.get` | Retrieve stored config | |
| `config.delete` | Delete stored config | **yes** |
| `config.urls` | List URLs under a config key | |

---

## Environment variable reference

### Server configuration

| Variable | Description | Default |
| --- | --- | --- |
| `LAB_MCP_HTTP_TOKEN` | Bearer token for HTTP API auth | (required for HTTP) |
| `LAB_MCP_TRANSPORT` | Default transport (`stdio` or `http`) | `stdio` |
| `LAB_MCP_HTTP_HOST` | HTTP bind host | `127.0.0.1` |
| `LAB_MCP_HTTP_PORT` | HTTP bind port | `8765` |
| `LAB_CORS_ORIGINS` | Additional CORS origins (comma-separated) | loopback only |
| `LAB_LOG` | Tracing filter directive | `lab=info,lab_apis=warn` |
| `LAB_LOG_FORMAT` | Log format (`json` for structured) | human |
| `LAB_ADMIN_ENABLED` | Enable `lab_admin` MCP tool | `0` |

### Per-service env vars

All services follow the pattern `{SERVICE}_{SUFFIX}`:

| Service | URL var | Auth var(s) | Default Port |
| --- | --- | --- | --- |
| Radarr | `RADARR_URL` | `RADARR_API_KEY` | 7878 |
| Sonarr | `SONARR_URL` | `SONARR_API_KEY` | 8989 |
| Prowlarr | `PROWLARR_URL` | `PROWLARR_API_KEY` | 9696 |
| Plex | `PLEX_URL` | `PLEX_TOKEN` | 32400 |
| Tautulli | `TAUTULLI_URL` | `TAUTULLI_API_KEY` | 8181 |
| SABnzbd | `SABNZBD_URL` | `SABNZBD_API_KEY` | 8080 |
| qBittorrent | `QBITTORRENT_URL` | `QBITTORRENT_PASSWORD` (+`USERNAME`, `SID`) | 8080 |
| Tailscale | — | `TAILSCALE_API_KEY` (+`TAILNET`, `BASE_URL`) | — |
| Linkding | `LINKDING_URL` | `LINKDING_TOKEN` | 9090 |
| Memos | `MEMOS_URL` | `MEMOS_API_KEY` | — |
| ByteStash | `BYTESTASH_URL` | `BYTESTASH_API_KEY` | — |
| Paperless | `PAPERLESS_URL` | `PAPERLESS_TOKEN` | 8000 |
| Arcane | `ARCANE_URL` | `ARCANE_API_KEY` | — |
| Unraid | `UNRAID_URL` | `UNRAID_API_KEY` | 31337 |
| UniFi | `UNIFI_URL` | `UNIFI_API_KEY` | 443 |
| Overseerr | `OVERSEERR_URL` | `OVERSEERR_API_KEY` | — |
| Gotify | `GOTIFY_URL` | `GOTIFY_TOKEN` (+`APP_TOKEN`, `CLIENT_TOKEN`) | 80 |
| OpenAI | `OPENAI_URL` (opt) | `OPENAI_API_KEY` (+`ORG_ID`) | — |
| Qdrant | `QDRANT_URL` | `QDRANT_API_KEY` (opt) | 6333 |
| TEI | `TEI_URL` | `TEI_API_KEY` | — |
| Apprise | `APPRISE_URL` | `APPRISE_API_KEY` | — |

**Multi-instance:** Append `_{LABEL}` before the suffix — `UNRAID_NODE2_URL`, `UNRAID_NODE2_API_KEY`.

---

## Feature flags

The `all` feature (default) enables all 22 services. Individual flags are 1:1 passthroughs from `lab` to `lab-apis`:

`radarr`, `sonarr`, `prowlarr`, `plex`, `tautulli`, `sabnzbd`, `qbittorrent`, `tailscale`, `linkding`, `memos`, `bytestash`, `paperless`, `arcane`, `unraid`, `unifi`, `overseerr`, `gotify`, `openai`, `qdrant`, `tei`, `apprise`

Build with a subset:

```bash
cargo build -p lab --features radarr,sonarr,plex
```

---

## Development

### Commands

```bash
just check            # cargo check --workspace --all-features
just test             # cargo nextest run --workspace --all-features
just test-integration # Run integration tests (requires running services)
just lint             # cargo clippy -D warnings + cargo fmt --check
just deny             # cargo deny check (licenses + vulnerabilities)
just build            # cargo build --workspace --all-features
just build-release    # cargo build --workspace --all-features --release
just run              # cargo run --all-features -- <args>
just fmt              # cargo fmt --all
just clean            # cargo clean
just release          # cargo release (version bump + tag + push)
just mcp-token        # Generate secure MCP bearer token
```

### Direct commands

```bash
cargo test -p lab-apis              # SDK client tests only (fast, wiremock)
cargo test -p lab                   # CLI/MCP/API tests only
cargo doc --no-deps --all-features  # Generate docs
```

**Authoritative verification path:**

```bash
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo test --workspace --tests --no-fail-fast --all-features
```

---

## Design highlights

- **Two-crate split:** SDK logic stays reusable and transport-agnostic.
- **One tool per service:** MCP stays compact (~23 tools) instead of exploding into hundreds.
- **Shared action catalog:** the same metadata drives CLI help, MCP discovery, HTTP action listings, and the TUI.
- **Three-surface dispatch:** CLI, MCP, and HTTP API all dispatch through the same shared layer — action semantics are defined once.
- **Feature-gated services:** compile only the services you need.
- **Destructive action safety:** elicitation (MCP), `-y` flag (CLI), `confirm: true` (HTTP API) — three confirmation paths for the same gate.
- **Structured error envelopes:** stable `kind` tags so agents can react programmatically, consistent across all surfaces.
- **Multi-instance services:** `{SERVICE}_{LABEL}_{SUFFIX}` env pattern for managing multiple instances of the same service.
- **Safe local mutation:** TUI `.mcp.json` patching uses validation, locking, backups, and atomic writes.
- **Privacy posture:** no telemetry, analytics, or phone-home behavior beyond explicit service calls.

## Docs map

Start with `docs/README.md`. The most useful topic docs:

- `docs/ARCH.md` — crate split, runtime surfaces, shared contracts
- `docs/SERVICES.md` — service inventory and feature model
- `docs/CLI.md` — CLI behavior and operator commands
- `docs/MARKETPLACE.md` — Claude Code plugin marketplace service
- `docs/MCP.md` — one-tool-per-service MCP design and envelopes
- `docs/CONFIG.md` — env/TOML ownership and multi-instance naming
- `docs/DISPATCH.md` — shared dispatch ownership and adapter direction
- `docs/ERRORS.md` — error taxonomy, stable kinds, envelope behavior
- `docs/OBSERVABILITY.md` — logging boundaries, required fields, correlation
- `docs/SERIALIZATION.md` — serde ownership, stable envelopes, output boundaries
- `docs/OPERATIONS.md` — CI, release, health, and repo workflows
- `docs/TESTING.md` — testing rules and verification expectations

## License

Workspace metadata declares `MIT OR Apache-2.0`.
