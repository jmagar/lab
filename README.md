# lab

`lab` is a Rust homelab control plane workspace. It contains a reusable SDK crate
(`lab-apis`) and one product binary (`lab`) that exposes service integrations and
operator capabilities through the CLI, an MCP server, an HTTP API, the Labby web UI,
and a Ratatui plugin manager.

The root README is the public entrypoint. The topic docs in [docs/](./docs/README.md)
are the source of truth for implementation contracts and operator workflows. If this
file and a topic doc disagree, update the owning topic doc and then refresh this file.

## What You Can Do

Lab is not only a service SDK. It is an operator console for a homelab and an AI-tooling
control plane.

| Feature Area | What Lab Provides |
| --- | --- |
| Marketplace browsing | Browse configured Claude Code and Codex plugin marketplaces, installed plugins, official MCP Registry servers, and ACP Registry agents from one catalog. Filter by type, inspect curated Lab metadata, sync the local MCP Registry mirror, and open installed artifact files from the web UI or API. |
| Stash workspaces | Open a marketplace plugin into a Lab-managed stash workspace under `~/.lab/stash`, edit files through the Labby UI, save changes, preview deploy diffs, and deploy the saved workspace back to the Claude Code or Codex target with explicit confirmation. |
| Artifact forks and updates | Track the marketplace artifact fork/update model in the action catalog: fork metadata, upstream/base snapshots, drift checks, update previews, update apply strategies, and AI merge suggestions. The lower-level direct `artifact.fork`, `artifact.diff`, and `artifact.patch` actions are present but still return `not_implemented` until that lifecycle is completed. |
| Device deployment and cherry-pick | Install whole plugins or cherry-pick individual skills, agents, slash commands, MCP server configs, scripts, and other plugin artifacts to any selected enrolled device and scope instead of copying files by hand. |
| MCP Registry search and install | Search, filter, validate, and install servers from the official MCP Registry. Installs can target Lab gateway upstreams or Claude/Codex MCP client configs on fleet devices, with required env values routed to the right config surface. |
| MCP Registry aggregator | Serve Lab's local registry mirror on `/v0.1/*` as a drop-in replacement for the official MCP Registry API, while layering Lab-owned metadata such as featured/reviewed/recommended flags, tags, audit fields, and homelab-specific curation without mutating upstream registry data. |
| ACP Registry agents | Search the ACP Registry for compatible agents, inspect agent details, and install or uninstall agent provider entries through the same marketplace service. ACP agent installs currently write controller-local provider config; remote ACP agent installation returns per-node errors until implemented. |
| Upstream MCP proxy | Point MCP clients at Lab instead of every individual server. Lab connects to configured HTTP or stdio upstream MCP servers, discovers their tools, optionally proxies resources, normalizes errors, applies circuit-breaker health, and republishes the merged catalog behind Lab's authenticated `/mcp` endpoint. |
| Gateway management | Add, test, reload, and remove upstream MCP servers without hand-editing config. Lab supports exposure filters, `tool_search`/`tool_invoke` helper paths, OAuth-backed upstream credentials, and Labby/CLI/API controls for the proxy pool. |
| Virtual Lab servers | Expose configured Lab-backed services as virtual gateway servers, toggle CLI/API/MCP/Web UI surfaces, inspect service action metadata, and set MCP action allowlists per virtual server. |
| Authentication and OAuth | Protect hosted HTTP, MCP, Labby, registry, and gateway-management routes with static bearer auth or Lab's Google-backed OAuth mode. Lab also supports browser sessions, CSRF-protected web UI calls, OAuth metadata/JWKS endpoints, upstream MCP OAuth credential storage, and local or node-started OAuth callback relays. |
| Fleet nodes | Run `lab serve` on multiple machines, enroll non-controller nodes, approve or deny devices, inspect node inventory and MCP client config metadata, run the local OAuth relay on a node, and route status/log events back to the controller over the fleet WebSocket. |
| Logs and activity | Search persisted local runtime logs, tail bounded history, stream live logs to the Labby `/logs` page over SSE, and forward peer syslog batches into the controller when enabled. |
| Labby web UI | Serve the admin UI from the same `lab serve` process as the API and MCP endpoint. The UI covers marketplace, gateway management, registry browsing, logs, setup, activity, settings, docs, and design-system/dev previews. |
| Labby chat | Use the `/chat` web UI as a live ACP client: create/list/resume sessions, send prompts to configured providers, stream session events over SSE, inspect transcript and reasoning/activity lanes, and render tool calls, terminal output, file trees, diffs, code blocks, links, and web previews. |
| TUI plugin manager | Run `lab plugins` to manage local service/plugin installation from a Ratatui interface that reads service metadata and patches `.mcp.json` entries without requiring hand-written MCP config. |
| Generated API docs and catalogs | Use `lab help --json`, MCP `lab://catalog`, per-service action resources, `/v1/{service}/actions`, `/v1/openapi.json`, and `/v1/docs` to discover the exact enabled action surface programmatically. |
| Workspace filesystem browser | Browse and preview files under the configured workspace root through the guarded `fs` service for Labby attachment and editor workflows. |
| Setup and health audits | Use `lab init`, `lab doctor`, `lab health`, `lab scaffold service`, and `lab audit onboarding` to bootstrap config, validate service reachability/auth, and keep new integrations aligned with the repo contract. |
| Service operations | Use one action catalog across CLI, MCP, and HTTP to operate Radarr, Sonarr, Plex, UniFi, Unraid, qBittorrent, Gotify, Qdrant, OpenAI-compatible APIs, and the rest of the service integrations. |
| Credential bootstrap | Scan local or SSH appdata paths with `lab extract`, preview diffs, and apply discovered service URLs/API keys into `~/.lab/.env` with backups and atomic writes. |
| Deployment and monitors | Build and push the Lab release binary to SSH targets, manage rollout policy, and use monitor definitions from `plugins/monitors/monitors.json` through `lab deploy monitor`. |

These features are exposed consistently:

- **CLI:** operator commands such as `lab marketplace`, `lab gateway`, `lab nodes`, `lab logs`, `lab doctor`, `lab deploy`, and per-service subcommands.
- **MCP:** compact one-tool-per-service access for agents, with generated action discovery and destructive-action confirmation.
- **HTTP/API:** `/v1/<service>` action dispatch, OpenAPI docs, OAuth/browser sessions, and same-origin Labby integration.
- **Web UI:** Labby pages for marketplace, gateways, logs, registry, setup, activity, and live ACP chat workflows.

## Current State

Fresh catalog source: `cargo run --all-features --bin lab -- help --json`, run from this
checkout on 2026-04-26.

| Metric | Value |
| --- | ---: |
| Registered services in the default all-features catalog | 30 |
| Callable actions in the default all-features catalog | 682 |
| Runtime opt-in admin service | `lab_admin` via `LAB_ADMIN_ENABLED=1` |
| Callable actions with `lab_admin` enabled | 685 |

The default all-features catalog registers these services:

| Service | Category | Actions | Description |
| --- | --- | ---: | --- |
| `extract` | bootstrap | 5 | Pull API keys and URLs from existing service config files |
| `gateway` | bootstrap | 34 | Manage proxied upstream MCP gateways |
| `doctor` | bootstrap | 6 | Comprehensive health audit: env vars, system probes, and service reachability |
| `logs` | bootstrap | 6 | Search and stream local-master runtime logs |
| `device` | bootstrap | 5 | Manage fleet device enrollments |
| `marketplace` | marketplace | 40 | Browse and install Claude Code plugins from configured marketplaces |
| `acp` | ai | 13 | Agent Client Protocol - session management and provider orchestration |
| `radarr` | servarr | 53 | Movie collection manager for Usenet and BitTorrent |
| `sonarr` | servarr | 34 | TV series management for the Servarr stack |
| `prowlarr` | indexer | 25 | Indexer manager for the Servarr stack |
| `plex` | media | 29 | Plex Media Server - library browsing, session management, and playlists |
| `tautulli` | media | 27 | Plex Media Server analytics - activity, history, statistics, and user management |
| `sabnzbd` | download | 22 | Usenet download client |
| `qbittorrent` | download | 31 | BitTorrent download client |
| `tailscale` | network | 24 | WireGuard-based mesh VPN - list devices, manage auth keys, query DNS settings |
| `linkding` | notes | 21 | Self-hosted bookmark manager with tagging and search |
| `memos` | notes | 20 | Lightweight self-hosted memo hub for short-form notes and journals |
| `bytestash` | notes | 18 | Self-hosted code snippet manager |
| `paperless` | documents | 31 | Self-hosted document management system with OCR and full-text search |
| `arcane` | network | 23 | Docker management UI - environments, containers, images, and volumes |
| `unraid` | network | 30 | Unraid server GraphQL API - system info, metrics, array status, Docker, and disk management |
| `unifi` | network | 81 | UniFi Network Application local API |
| `overseerr` | media | 30 | Request and approval frontend for Plex, Sonarr, and Radarr |
| `gotify` | notifications | 26 | Self-hosted push notification server |
| `openai` | ai | 6 | OpenAI API: chat, embeddings, models, images, audio |
| `qdrant` | ai | 15 | Vector database for similarity search and RAG |
| `tei` | ai | 10 | Hugging Face TEI server - embeddings, rerankers, sequence classification |
| `apprise` | notifications | 10 | Universal notification dispatcher: 100+ services behind one URL scheme |
| `deploy` | bootstrap | 6 | Build and push the lab release binary to SSH targets with integrity verification |
| `fs` | bootstrap | 1 | Workspace filesystem browser: read-only, deny-listed |

`lab_admin` is compiled by the default `all` feature but only registers at runtime when
`LAB_ADMIN_ENABLED=1` or `[admin].enabled = true`.

## Workspace Layout

| Path | Role |
| --- | --- |
| [crates/lab-apis](./crates/lab-apis) | Pure Rust SDK: typed clients, request/response models, auth, shared HTTP behavior, health contracts, plugin metadata |
| [crates/lab](./crates/lab) | Product binary: CLI, MCP, HTTP API, TUI, config loading, dispatch, output rendering, catalog generation |
| [crates/lab-auth](./crates/lab-auth) | HTTP/OAuth auth support used by the hosted runtime |
| [apps/gateway-admin](./apps/gateway-admin/README.md) | Labby admin UI, exported and served by `lab serve` when static assets exist |
| [docs](./docs/README.md) | Topic-based source-of-truth documentation |
| [plugins](./plugins) | Lab plugin and monitor assets used by local workflows |

Core boundary: shared service logic belongs in `lab-apis`; product surfaces and adapters
belong in `lab`.

## Quick Start

The workspace uses Rust 2024 and the root toolchain requirement currently resolves to
Rust 1.90+.

```bash
git clone git@github.com:jmagar/lab.git
cd lab
cargo build --workspace --all-features
cargo install --path crates/lab --all-features
```

Secrets and endpoint URLs belong in `~/.lab/.env`. Preferences belong in
`config.toml`, searched in this order: `./config.toml`, `~/.lab/config.toml`,
`~/.config/lab/config.toml`.

Startup reads the first `config.toml`, initializes logging, then loads
`~/.lab/.env` and a CWD `.env` if present. Runtime value precedence is still:

Value precedence is:

1. CLI flags
2. Process environment, including values loaded from `~/.lab/.env`
3. The first `config.toml` found
4. Built-in defaults

Minimal `~/.lab/.env`:

```env
LAB_MCP_HTTP_TOKEN=replace-with-openssl-rand-hex-32

RADARR_URL=http://localhost:7878
RADARR_API_KEY=abc123

SONARR_URL=http://localhost:8989
SONARR_API_KEY=abc123
```

Minimal `~/.lab/config.toml`:

```toml
[mcp]
transport = "http"
host = "127.0.0.1"
port = 8765

[api]
cors_origins = []

[auth]
mode = "bearer"
```

Use [config.example.toml](./config.example.toml), [.env.example](./.env.example),
[docs/CONFIG.md](./docs/CONFIG.md), and [docs/ENV.md](./docs/ENV.md) for the full
configuration contract.

## Runtime Commands

```bash
lab help
lab help --json
lab serve
lab serve --host 127.0.0.1 --port 8765
lab serve --services radarr,sonarr,plex
lab serve mcp --stdio
lab doctor
lab health
lab plugins
lab extract /mnt/appdata --diff
```

`lab serve` defaults to the hosted runtime path. Unless transport is explicitly set to
stdio, it starts the Axum HTTP server for the product API, OAuth/auth endpoints when
configured, the HTTP MCP surface at `/mcp`, and the Labby web UI when exported assets
exist.

`lab serve mcp --stdio` is the stdio-only MCP path for local editor and desktop clients.
It does not start the hosted API or web UI.

## Auth And OAuth

Hosted Lab surfaces are protected by the same auth layer, with deliberately separate
rules for local development, HTTP MCP clients, browser sessions, upstream gateways, and
fleet node enrollment.

HTTP auth modes:

| Mode | Required config | Notes |
| --- | --- | --- |
| Bearer | `LAB_AUTH_MODE=bearer` or default, plus `LAB_MCP_HTTP_TOKEN` for protected deployments | Uses constant-time static bearer-token comparison |
| OAuth | `LAB_AUTH_MODE=oauth`, `LAB_PUBLIC_URL`, `LAB_GOOGLE_CLIENT_ID`, `LAB_GOOGLE_CLIENT_SECRET` | Enables Lab's Google-backed auth server, JWT validation, metadata, browser sessions, and callback handling |

Protected route behavior:

| Surface | Accepted auth | Notes |
| --- | --- | --- |
| `/v1/*` product API | Static bearer token, Lab OAuth JWT bearer token, or Labby browser session cookie | Browser session POSTs use CSRF protection. `LAB_WEB_UI_DISABLE_AUTH` bypasses `/v1` auth only for development. |
| `/mcp` HTTP MCP | Static bearer token or Lab OAuth JWT bearer token | Browser session cookies are not accepted for MCP transport. |
| `/v0.1/*` MCP Registry compatibility routes | Same as protected `/v1` routes | Mounted when the registry feature is enabled. |
| Labby web UI | Browser session in OAuth mode, or the configured development bypass | Static assets and SPA paths are served by `lab serve`; data calls still use `/v1`. |
| `/health`, `/ready` | No auth | Intended for probes. |
| `/v1/nodes/hello`, `/v1/nodes/ws` | No bearer middleware | Node WebSocket `initialize` validates the enrolled `device_id` and device token before node methods run. |

OAuth mode exposes `/.well-known/oauth-authorization-server`,
`/.well-known/oauth-protected-resource`, `/jwks`, `/register`, `/authorize`,
`/token`, `/auth/login`, `/auth/session`, `/auth/logout`, and
`/auth/google/callback`. Lab stores Google tokens server-side and issues Lab JWTs for
API/MCP clients.

Gateway OAuth is separate from Lab login OAuth. Upstreams configured with
`[upstream.oauth]` use `/v1/gateway/oauth/start`, `/auth/upstream/callback`,
`/v1/gateway/oauth/status`, and `/v1/gateway/oauth/clear`; credentials are encrypted in
Lab's auth store and shared by the web UI, CLI, and MCP gateway actions.

Callback relay helpers cover split-browser/device flows:

```bash
lab oauth relay-local --machine dookie --port 38935
lab oauth relay-local --forward-base http://100.88.16.79:38935/callback/dookie --port 38935
```

The same local relay can be started through the node runtime with
`POST /v1/nodes/oauth/relay/start`. Relays forward the final callback request only; they
do not mint tokens, store PKCE state, or complete the OAuth exchange.

When binding HTTP to a non-loopback host, configure bearer or OAuth auth. The server warns
and refuses unsafe exposed configurations. See [docs/OAUTH.md](./docs/OAUTH.md),
[docs/TRANSPORT.md](./docs/TRANSPORT.md), and [docs/GATEWAY.md](./docs/GATEWAY.md) for
the full auth contract.

## CLI

Top-level commands are defined in [crates/lab/src/cli.rs](./crates/lab/src/cli.rs):

| Command | Purpose |
| --- | --- |
| `lab serve` | Start the hosted runtime or stdio MCP server |
| `lab help` | Print the generated service and action catalog |
| `lab doctor` | Run comprehensive health audits |
| `lab health` | Run quick reachability checks |
| `lab nodes` | Query nodes from the configured controller |
| `lab logs` | Search fleet or local-master logs |
| `lab plugins` | Open the Ratatui plugin manager |
| `lab marketplace` | Manage Claude Code, Codex, MCP Registry, and ACP Registry marketplace entries |
| `lab gateway` | Manage proxied upstream MCP gateways |
| `lab oauth` | Run local OAuth callback relay helpers |
| `lab extract` | Scan local or SSH appdata paths and extract service credentials |
| `lab audit` | Audit service onboarding against the repo contract |
| `lab scaffold` | Generate a new service onboarding scaffold |
| `lab install` / `lab uninstall` | Patch `.mcp.json` service entries |
| `lab init` | Run first-time setup |
| `lab completions` | Generate shell completions |

Feature-gated services also expose CLI subcommands such as `lab radarr`, `lab unifi`,
`lab qdrant`, and `lab deploy`.

CLI output is human-readable by default. Use global `--json` for machine-readable output
and `--color auto|always|never` for human output styling. See [docs/CLI.md](./docs/CLI.md)
and [docs/design/CLI_DESIGN_SYSTEM.md](./docs/design/CLI_DESIGN_SYSTEM.md).

Destructive CLI operations require confirmation. Non-interactive callers use `-y` or
`--yes` where the subcommand exposes it; dry-run capable paths document `--dry-run`.

## MCP Server

Core Lab services use one MCP tool per registered service. Gateway and upstream features
can add healthy upstream tools plus optional `tool_search` and `tool_invoke` helpers.
Core service tool input is:

```json
{
  "action": "movie.search",
  "params": { "query": "Inception" }
}
```

Discovery surfaces:

| Surface | Purpose |
| --- | --- |
| `lab.help` MCP meta-tool | Full service and action catalog |
| `lab://catalog` MCP resource | Full generated catalog |
| `lab://<service>/actions` MCP resource | Per-service action list |
| `help` action | Per-tool action catalog |
| `schema` action | Parameter schema for one action |

Upstream MCP resources may be merged into the resource list when upstream gateway
proxying is configured.

Destructive MCP actions use elicitation when the client supports it. Headless clients pass
`"confirm": true` inside `params`; otherwise the tool returns `confirmation_required`.

See [docs/MCP.md](./docs/MCP.md), [docs/RMCP.md](./docs/RMCP.md), and
[docs/TRANSPORT.md](./docs/TRANSPORT.md).

## HTTP API

`lab serve` mounts the HTTP API under `/v1` and MCP over HTTP at `/mcp` in hosted mode.
Protected routes accept configured auth: static bearer tokens, OAuth JWT bearer tokens,
and browser session cookies on `/v1` routes. Loopback development can run without auth
when neither bearer nor OAuth auth is configured; non-loopback binds require configured auth.

Unauthenticated routes include:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/ready` | Readiness |
| `POST` | `/v1/nodes/hello` | Public node self-registration |
| `GET` | `/v1/nodes/ws` | Node WebSocket upgrade; JSON-RPC session validates enrollment |
| `GET` | `/` and SPA paths | Labby web UI when exported assets are available |
| `GET` / `POST` | OAuth metadata and auth paths | Mounted when OAuth auth state is configured |

Protected routes include:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/v1/{service}/actions` | List generated actions for a service |
| `POST` | `/v1/{service}` | Dispatch one action with `action` and `params` |
| `GET` | `/v1/openapi.json` | OpenAPI 3.1 spec |
| `GET` | `/v1/docs` | Scalar API docs UI |
| `GET` / `POST` | `/v0.1/*` | MCP Registry compatibility routes when the feature is enabled |

Example:

```bash
curl -s -X POST http://127.0.0.1:8765/v1/radarr \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"movie.search","params":{"query":"Inception"}}'
```

Destructive HTTP actions require `"confirm": true` in `params`. Missing confirmation
returns `422` with `kind: "confirmation_required"`. Error responses use the shared
structured envelope and stable `kind` vocabulary from [docs/ERRORS.md](./docs/ERRORS.md).

Middleware order is request id, tracing, request-id propagation, timeout, compression, and
CORS. Loopback origins are allowed by default; add more origins with `LAB_CORS_ORIGINS` or
`[api].cors_origins`.

See [docs/TRANSPORT.md](./docs/TRANSPORT.md), [docs/OAUTH.md](./docs/OAUTH.md),
[docs/ERRORS.md](./docs/ERRORS.md), and [docs/OBSERVABILITY.md](./docs/OBSERVABILITY.md).

## Service Catalogs

Do not maintain action lists by hand in this README. The source-of-truth catalog is generated
from the registry and action specs:

```bash
lab help --json
lab help
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  http://127.0.0.1:8765/v1/radarr/actions
```

Action-level coverage docs live under [docs/coverage](./docs/coverage):

| Service | Coverage Doc |
| --- | --- |
| Apprise | [docs/coverage/apprise.md](./docs/coverage/apprise.md) |
| Arcane | [docs/coverage/arcane.md](./docs/coverage/arcane.md) |
| ByteStash | [docs/coverage/bytestash.md](./docs/coverage/bytestash.md) |
| Gotify | [docs/coverage/gotify.md](./docs/coverage/gotify.md) |
| Linkding | [docs/coverage/linkding.md](./docs/coverage/linkding.md) |
| MCP Registry | [docs/coverage/mcpregistry.md](./docs/coverage/mcpregistry.md) |
| Memos | [docs/coverage/memos.md](./docs/coverage/memos.md) |
| OpenAI | [docs/coverage/openai.md](./docs/coverage/openai.md) |
| Overseerr | [docs/coverage/overseerr.md](./docs/coverage/overseerr.md) |
| Paperless | [docs/coverage/paperless.md](./docs/coverage/paperless.md) |
| Plex | [docs/coverage/plex.md](./docs/coverage/plex.md) |
| Prowlarr | [docs/coverage/prowlarr.md](./docs/coverage/prowlarr.md) |
| qBittorrent | [docs/coverage/qbittorrent.md](./docs/coverage/qbittorrent.md) |
| Qdrant | [docs/coverage/qdrant.md](./docs/coverage/qdrant.md) |
| Radarr | [docs/coverage/radarr.md](./docs/coverage/radarr.md) |
| SABnzbd | [docs/coverage/sabnzbd.md](./docs/coverage/sabnzbd.md) |
| Sonarr | [docs/coverage/sonarr.md](./docs/coverage/sonarr.md) |
| Tailscale | [docs/coverage/tailscale.md](./docs/coverage/tailscale.md) |
| Tautulli | [docs/coverage/tautulli.md](./docs/coverage/tautulli.md) |
| TEI | [docs/coverage/tei.md](./docs/coverage/tei.md) |
| UniFi | [docs/coverage/unifi.md](./docs/coverage/unifi.md) |
| Unraid | [docs/coverage/unraid.md](./docs/coverage/unraid.md) |

## Environment Reference

Service credentials follow the pattern `{SERVICE}_URL`, `{SERVICE}_API_KEY`,
`{SERVICE}_TOKEN`, `{SERVICE}_USERNAME`, and `{SERVICE}_PASSWORD`. Multi-instance
services insert the label before the suffix: `UNRAID_NODE2_URL`,
`UNRAID_NODE2_API_KEY`.

| Service | Required Env | Optional Env | Default Port |
| --- | --- | --- | ---: |
| Radarr | `RADARR_URL`, `RADARR_API_KEY` |  | 7878 |
| Sonarr | `SONARR_URL`, `SONARR_API_KEY` |  | 8989 |
| Prowlarr | `PROWLARR_URL`, `PROWLARR_API_KEY` |  | 9696 |
| Plex | `PLEX_URL`, `PLEX_TOKEN` |  | 32400 |
| Tautulli | `TAUTULLI_URL`, `TAUTULLI_API_KEY` |  | 8181 |
| SABnzbd | `SABNZBD_URL`, `SABNZBD_API_KEY` |  | 8080 |
| qBittorrent | `QBITTORRENT_URL`, `QBITTORRENT_PASSWORD` | `QBITTORRENT_USERNAME`, `QBITTORRENT_SID` | 8080 |
| Tailscale | `TAILSCALE_API_KEY` | `TAILSCALE_TAILNET`, `TAILSCALE_BASE_URL` |  |
| Linkding | `LINKDING_URL`, `LINKDING_TOKEN` |  | 9090 |
| Memos | `MEMOS_URL`, `MEMOS_TOKEN` |  | 5230 |
| ByteStash | `BYTESTASH_URL`, `BYTESTASH_TOKEN` |  | 5000 |
| Paperless | `PAPERLESS_URL`, `PAPERLESS_TOKEN` |  | 8000 |
| Arcane | `ARCANE_URL`, `ARCANE_API_KEY` |  | 3000 |
| Unraid | `UNRAID_URL`, `UNRAID_API_KEY` |  | 31337 |
| UniFi | `UNIFI_URL`, `UNIFI_API_KEY` | `UNIFI_RESOLVE_IP`, `UNIFI_ALLOW_INSECURE_TLS` | 443 |
| Overseerr | `OVERSEERR_URL`, `OVERSEERR_API_KEY` |  | 5055 |
| Gotify | `GOTIFY_URL` | `GOTIFY_TOKEN`, `GOTIFY_APP_TOKEN`, `GOTIFY_CLIENT_TOKEN` | 80 |
| OpenAI | `OPENAI_API_KEY` | `OPENAI_URL`, `OPENAI_ORG_ID` |  |
| Qdrant | `QDRANT_URL` | `QDRANT_API_KEY` | 6333 |
| TEI | `TEI_URL` | `TEI_API_KEY` | 80 |
| Apprise | `APPRISE_URL` | `APPRISE_TOKEN` | 8000 |
| ACP |  | `LAB_ACP_DB`, `LAB_ACP_HMAC_SECRET` |  |
| MCP Registry |  | `MCPREGISTRY_URL` |  |
| ACP Registry |  | `ACP_REGISTRY_URL` |  |

Server and runtime env:

| Variable | Purpose |
| --- | --- |
| `LAB_MCP_HTTP_TOKEN` | Static bearer token for protected HTTP routes and HTTP MCP |
| `LAB_MCP_TRANSPORT` | Default transport, `http` or `stdio` |
| `LAB_MCP_HTTP_HOST` / `LAB_MCP_HTTP_PORT` | Hosted runtime bind address |
| `LAB_MCP_SESSION_TTL_SECS` / `LAB_MCP_STATEFUL` | HTTP MCP session behavior |
| `LAB_MCP_ALLOWED_HOSTS` | Additional DNS-rebinding allowed hosts |
| `LAB_CORS_ORIGINS` | Additional browser CORS origins |
| `LAB_AUTH_MODE` | `bearer` or `oauth` |
| `LAB_PUBLIC_URL` | Public base URL for OAuth metadata, JWT issuer/audience, callback construction, and allowed-host derivation |
| `LAB_GOOGLE_CLIENT_ID` / `LAB_GOOGLE_CLIENT_SECRET` | Google OAuth credentials for OAuth mode |
| `LAB_GOOGLE_CALLBACK_PATH` | Optional Google callback path override |
| `LAB_AUTH_ALLOWED_REDIRECT_URIS` | Optional non-loopback MCP OAuth callback allowlist |
| `LAB_WEB_ASSETS_DIR` | Static Labby export directory override |
| `LAB_WEB_UI_DISABLE_AUTH` | Development-only bypass for Labby browser auth |
| `LAB_LOG` / `LAB_LOG_FORMAT` | Tracing filter and text/json log format |
| `LAB_ADMIN_ENABLED` | Runtime opt-in for the `lab_admin` MCP tool |

## Feature Flags

`lab` defaults to `all`. `lab-apis` defaults to no optional upstream services.

Feature-gated upstream integrations:

`radarr`, `sonarr`, `prowlarr`, `overseerr`, `plex`, `tautulli`, `sabnzbd`,
`qbittorrent`, `tailscale`, `unraid`, `unifi`, `arcane`, `linkding`, `memos`,
`bytestash`, `paperless`, `gotify`, `apprise`, `openai`, `qdrant`, `tei`,
`deploy`, `mcpregistry`, `acp_registry`, `fs`, `lab-admin`.

Always-on product/capability services include `extract`, `gateway`, `doctor`, `logs`,
`device`, `marketplace`, and `acp`. `lab_admin` is feature-enabled by `all` but
runtime-gated.

Build a subset:

```bash
cargo build -p lab --no-default-features --features radarr,sonarr,plex
```

## Development

Use the `just` aliases when possible:

```bash
just check            # cargo check --workspace --all-features
just test             # cargo nextest run --workspace --all-features
just test-integration # cargo nextest run --workspace --all-features -- --ignored
just lint             # cargo clippy --workspace --all-features -- -D warnings; cargo fmt --all -- --check
just deny             # cargo deny check
just build            # cargo build --workspace --all-features
just build-release    # cargo build --workspace --all-features --release
just web-build        # cd apps/gateway-admin && pnpm build
just web-watch        # rebuild Labby static assets on frontend changes
just run -- help      # cargo run --all-features -- <args>
just chat-local       # local Labby chat workflow with auth disabled for development
just fmt              # cargo fmt --all
just clean            # cargo clean
just release          # cargo release
just mcp-token        # rotate LAB_MCP_HTTP_TOKEN in .env
```

Authoritative verification is all-features:

```bash
cargo check --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo build --workspace --all-features
```

Use `cargo test` only for narrow local slices or when a tool specifically requires it.
The repo-level test contract is [docs/TESTING.md](./docs/TESTING.md).

## Docs Map

Start at [docs/README.md](./docs/README.md). Topic ownership:

| Doc | Owns |
| --- | --- |
| [docs/ARCH.md](./docs/ARCH.md) | Crate split, runtime surfaces, shared contracts, runtime flow |
| [docs/TECH.md](./docs/TECH.md) | Stack choices, toolchain, feature posture, verification surfaces, release tooling |
| [docs/CONVENTIONS.md](./docs/CONVENTIONS.md) | Locked engineering rules, async style, HTTP, testing, docs, privacy |
| [docs/SERVICES.md](./docs/SERVICES.md) | Service inventory, feature gates, metadata, multi-instance model |
| [docs/SERVICE_ONBOARDING.md](./docs/SERVICE_ONBOARDING.md) | End-to-end checklist for adding a service |
| [docs/SCAFFOLD_AND_AUDIT.md](./docs/SCAFFOLD_AND_AUDIT.md) | `lab scaffold service` and `lab audit onboarding` contract |
| [docs/CLI.md](./docs/CLI.md) | CLI behavior, command rules, confirmations, operator commands |
| [docs/design/CLI_DESIGN_SYSTEM.md](./docs/design/CLI_DESIGN_SYSTEM.md) | Human-readable CLI output language and color policy |
| [docs/design/CLI_OUTPUT_THEME_API.md](./docs/design/CLI_OUTPUT_THEME_API.md) | Proposed Rust API for CLI semantic styling |
| [docs/MCP.md](./docs/MCP.md) | MCP transport model, one-tool-per-service design, discovery, envelopes |
| [docs/RMCP.md](./docs/RMCP.md) | RMCP SDK integration contract |
| [docs/TRANSPORT.md](./docs/TRANSPORT.md) | Stdio and streamable HTTP transport, sessions, CORS, DNS rebinding |
| [docs/OAUTH.md](./docs/OAUTH.md) | Bearer vs OAuth auth, Google flow, JWTs, JWKS, metadata, callback forwarding |
| [docs/CONFIG.md](./docs/CONFIG.md) | Env/TOML ownership, load order, secrets, instance naming |
| [docs/ENV.md](./docs/ENV.md) | Deployment-ready env examples and auth mode variables |
| [docs/ERRORS.md](./docs/ERRORS.md) | Stable error taxonomy, envelopes, status mapping |
| [docs/design/SERIALIZATION.md](./docs/design/SERIALIZATION.md) | Serde ownership, stable envelopes, output-boundary rules |
| [docs/DISPATCH.md](./docs/DISPATCH.md) | Surface-neutral dispatch ownership and adapter direction |
| [docs/SERVICE_LAYER_MIGRATION.md](./docs/SERVICE_LAYER_MIGRATION.md) | Migration phases for shared dispatch/service layer |
| [docs/OBSERVABILITY.md](./docs/OBSERVABILITY.md) | Logging boundaries, required fields, correlation, redaction, verification |
| [docs/OPERATIONS.md](./docs/OPERATIONS.md) | Repo helpers, doctor/health workflows, CI, releases, updates |
| [docs/CICD.md](./docs/CICD.md) | GitHub Actions check matrix and release behavior |
| [docs/TESTING.md](./docs/TESTING.md) | Test runner contract and verification expectations |
| [docs/EXTRACT.md](./docs/EXTRACT.md) | Bootstrap credential extraction and `.env` merge semantics |
| [docs/GATEWAY.md](./docs/GATEWAY.md) | Upstream MCP gateway CRUD, reload/test flows, exposure policy |
| [docs/UPSTREAM.md](./docs/UPSTREAM.md) | Upstream MCP proxy setup, tool merging, circuit breaker, resources |
| [docs/MARKETPLACE.md](./docs/MARKETPLACE.md) | Marketplace service, plugin workspace mirrors, save/deploy flows |
| [docs/MCPREGISTRY_METADATA.md](./docs/MCPREGISTRY_METADATA.md) | Lab-owned metadata layered onto MCP Registry entries |
| [docs/acp/README.md](./docs/acp/README.md) | ACP service architecture and chat boundary |
| [docs/acp/design.md](./docs/acp/design.md) | ACP design details |
| [docs/acp/research-findings.md](./docs/acp/research-findings.md) | ACP research notes |
| [docs/DEVICE_RUNTIME.md](./docs/DEVICE_RUNTIME.md) | Master/non-master runtime roles and device inventory |
| [docs/NODES.md](./docs/NODES.md) | Node-facing CLI/API behavior |
| [docs/NODE_RUNTIME_CONTRACT.md](./docs/NODE_RUNTIME_CONTRACT.md) | Controller/node split and node artifact rules |
| [docs/FLEET_METHODS.md](./docs/FLEET_METHODS.md) | Fleet WebSocket JSON-RPC method contract |
| [docs/FLEET_LOGS.md](./docs/FLEET_LOGS.md) | Fleet log ingestion, queueing, search, storage limits |
| [docs/LOCAL_LOGS.md](./docs/LOCAL_LOGS.md) | Local-master runtime log store, `/v1/logs`, SSE streaming |
| [docs/DEPLOY.md](./docs/DEPLOY.md) | Device-runtime deployment topology and rollout guidance |
| [docs/DEPLOY_SERVICE.md](./docs/DEPLOY_SERVICE.md) | Deploy service action/API contract |
| [docs/MONITORS.md](./docs/MONITORS.md) | Claude Code monitor definitions and `lab deploy monitor` |
| [docs/TUI.md](./docs/TUI.md) | Ratatui plugin manager behavior and `.mcp.json` patching |
| [apps/gateway-admin/README.md](./apps/gateway-admin/README.md) | Labby frontend workflow and static export model |
| [docs/design/component-development.md](./docs/design/component-development.md) | Labby component workflow and browser verification |
| [docs/design/design-system-contract.md](./docs/design/design-system-contract.md) | Labby Aurora design-system contract |
| [docs/design/CLAUDE_CODE_AURORA_THEME.md](./docs/design/CLAUDE_CODE_AURORA_THEME.md) | Aurora theme mapping for Claude Code-like surfaces |

Supporting directories:

| Path | Purpose |
| --- | --- |
| [docs/coverage](./docs/coverage) | Per-service coverage and action mapping |
| [docs/upstream-api](./docs/upstream-api/README.md) | Upstream specs and reference material |
| [docs/features](./docs/features/FEATURE_BRIEF.md) | Product feature briefs and implementation notes |
| [docs/reviews](./docs/reviews) | Review artifacts |
| [docs/reports](./docs/reports) | Verification and audit reports |
| [docs/sessions](./docs/sessions) | Historical session notes |
| [docs/superpowers/plans](./docs/superpowers/plans) | Historical implementation plans |
| [docs/superpowers/specs](./docs/superpowers/specs) | Historical implementation specs |
| [docs/mockups](./docs/mockups) | Static UI mockups |

## Design Highlights

- One binary exposes many integrations without exploding the MCP tool list.
- The generated action catalog drives CLI help, MCP discovery, HTTP action listings, and UI surfaces.
- CLI, MCP, and HTTP dispatch share action semantics instead of duplicating business logic.
- Destructive actions have a single metadata flag and separate confirmations per surface.
- Structured error envelopes use stable `kind` tags across agent-facing surfaces.
- Multi-instance service selection is a config-layer concern, not hardcoded per service.
- Labby is served from the same hosted runtime when static assets exist; the separate Next dev server is only for frontend development.
- No telemetry or analytics are built in; network calls are explicit service/API operations.

## License

Workspace metadata declares `MIT OR Apache-2.0`.
