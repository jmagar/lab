# Services

`lab` is built around feature-gated service integrations plus a small number of product-local surfaces. Most integrations follow the same structural contract so the CLI, MCP server, API, and TUI can treat them uniformly.

## Per-Service Shape

Most service integrations provide:

- a `lab-apis` module
- a typed client
- request/response types
- a service-specific error type
- a shared dispatch entry
- a `PluginMeta`
- a health-check implementation
- a CLI shim
- an MCP dispatch shim
- an API shim when the service is exposed over HTTP

Product-local surfaces are split into two categories:

- product-local control-plane surfaces, which may live entirely in `lab` when
  they primarily coordinate runtime behavior inside the product
- product-local capability modules, whose core logic still belongs in
  `lab-apis` even when they do not wrap a conventional upstream HTTP API

`gateway` is the reference control-plane surface and is allowed to live
entirely in `lab`.

`extract` is the reference non-HTTP capability module and keeps its core logic
in `lab-apis`.

The ACP/chat work should follow the capability-module pattern for ACP itself:
- `acp` becomes the first-class capability/service
- `chat` remains the UI route and presentation layer over that service

## Feature Gates

Most upstream-backed service integrations are feature-gated in `lab-apis` and re-exported in `lab`.

Categories currently include:

- media
- Servarr
- indexers
- downloads
- notes and documents
- network and infrastructure
- notifications
- AI and inference

Default feature posture:

- `lab-apis` defaults to no optional services
- `lab` defaults to all service integrations
- `all` enables every service integration
- `extract` remains always available
- `stash` remains always available (local versioning service, no feature gate)

## Always-On Services

The following services are always compiled in and require no feature flag or upstream service:

| Service | Description | Docs |
|---------|-------------|------|
| `extract` | Scan local and SSH hosts for service credentials | [EXTRACT.md](./EXTRACT.md) |
| `device_runtime` | Local device runtime introspection | [DEVICE_RUNTIME.md](./DEVICE_RUNTIME.md) |
| `stash` | Local versioned component snapshots with provider sync | [STASH.md](./STASH.md) |

## Service Sources

### Services With OpenAPI Specs

- Radarr
- Sonarr
- Prowlarr
- Overseerr
- Plex
- Jellyfin (`docs/upstream-api/jellyfin.openapi.json`)
- Tailscale
- Paperless-ngx
- Memos
- ByteStash
- Arcane
- Gotify
- OpenAI
- Qdrant
- TEI
- MCP Registry (`docs/upstream-api/mcp-registry.yaml`)
- Immich (`docs/upstream-api/immich.md`; upstream OpenAPI-backed docs at https://api.immich.app/)
- AdGuard Home (`docs/upstream-api/adguard.md`; upstream OpenAPI contract)
- Pi-hole (`docs/upstream-api/pihole.md`; v6 REST API with session auth)

### Services Without OpenAPI Specs

- Apprise
- Tautulli
- SABnzbd
- qBittorrent
- Linkding
- Unraid
- UniFi
- NotebookLM (`teng-lin/notebooklm-py` RPC contract)
- FreshRSS (`docs/upstream-api/freshrss.md`; Google Reader-compatible API)
- Navidrome (`docs/upstream-api/navidrome.md`; Subsonic/OpenSubsonic-compatible API)
- Scrutiny (`docs/upstream-api/scrutiny.md`; cautious read-only endpoint contract)
- LoggiFly (`docs/upstream-api/loggifly.md`; implementation-deferred contract status only)
- Glances (`docs/upstream-api/glances.md`; REST API v4)
- Uptime Kuma (`docs/upstream-api/uptime-kuma.md`; Socket.IO-backed monitor API, live reads deferred)

### Deferred

- Radicale
- LoggiFly runtime config/health integration beyond `contract.status`
- Uptime Kuma live Socket.IO monitor reads beyond `contract.status` and `server.health`

These are implemented from vendor docs, reference docs, or hand-written API contracts.

## Plugin Metadata

Every service publishes `PluginMeta` alongside the service module.

That metadata drives:

- TUI grouping
- install/uninstall prompts
- required env validation
- doctor checks
- docs and presentation

Metadata includes:

- canonical service name
- display name
- short description
- category
- docs URL
- required env vars
- optional env vars
- default port

Categories are part of the product model:

- `Media`
- `Servarr`
- `Indexer`
- `Download`
- `Notes`
- `Documents`
- `Network`
- `Notifications`
- `Ai`
- `Bootstrap`

## Multi-Instance Support

Multi-instance support is generic rather than hardcoded per service.

The config layer recognizes:

- `SERVICE_URL` as the default instance
- `SERVICE_<LABEL>_URL` as named instances

This is especially relevant for:

- Unraid
- Jellyfin
- OpenACP
- Plex
- qBittorrent
- any user who runs multiple copies of the same service

The service library layer stays unaware of instance naming. Instance lookup is a binary-level config concern.

OpenACP is registered as `openacp` and represents the upstream OpenACP daemon,
not Lab's internal `acp` service. Its actions intentionally stay
non-destructive in Lab's action catalog, so Lab CLI/MCP/API confirmation gates
do not apply to prompt/session, config, topic, tunnel, notify, or restart
actions.

## Adding a New Service

Use [SERVICE_ONBOARDING.md](./SERVICE_ONBOARDING.md) as the authoritative end-to-end checklist.

At a high level:

1. start from the upstream spec in `docs/upstream-api/`
2. build the `lab-apis` client and types
3. wire CLI, MCP, and HTTP shims
4. register the service in feature flags, discovery, dispatch, and metadata
5. update the coverage doc under `docs/coverage/`
6. test locally and verify against a real instance when possible

The important rule is that the service client owns logic. CLI, MCP, and HTTP layers only adapt inputs and outputs.

## Service Inventory Direction

The project is intentionally broad but follows one rule: one binary, one consistent control plane, many integrations.

The service set is grouped conceptually, not implemented as unrelated one-offs.

## Synthetic Service

[`EXTRACT.md`](./EXTRACT.md) documents `extract`, which is not a remote API
service but still follows the shared dispatch model for consistency.
[`GATEWAY.md`](./GATEWAY.md) documents a product-local management surface that
edits and reloads `[[upstream]]` config and therefore does not fit the usual
`lab-apis` service shape. [`acp/README.md`](./acp/README.md) documents ACP as a
product-local capability service whose core logic belongs in `lab-apis` while
its adapters and registration live in `lab`.

## Chat / ACP Surface

The `/chat` experience is currently a product-local UI surface rather than a first-class service integration:

- it is wired to the gateway backend and ACP bridge endpoints
- its behavior lives in `apps/gateway-admin` plus supporting Rust API routes
- it does not yet have the full first-class `acp` service shape
- the intended promotion path is `acp` as the first-class service and `chat` as
  the UI over it

If we promote chat to a service later, it should follow `SERVICE_ONBOARDING.md` and `DISPATCH.md` like any other first-class integration.
