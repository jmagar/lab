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

Product-local surfaces such as `gateway` are allowed to live entirely in `lab` when they manage product behavior rather than wrapping an upstream service API.

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

## Service Sources

### Services With OpenAPI Specs

- Radarr
- Sonarr
- Prowlarr
- Overseerr
- Plex
- Tailscale
- Paperless-ngx
- Memos
- ByteStash
- Arcane
- Gotify
- OpenAI
- Qdrant
- TEI

### Services Without OpenAPI Specs

- Apprise
- Tautulli
- SABnzbd
- qBittorrent
- Linkding
- Unraid
- UniFi

### Deferred

- Radicale

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
- Plex
- qBittorrent
- any user who runs multiple copies of the same service

The service library layer stays unaware of instance naming. Instance lookup is a binary-level config concern.

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

[`EXTRACT.md`](./EXTRACT.md) documents `extract`, which is not a remote API service but still follows the shared dispatch model for consistency. [`GATEWAY.md`](./GATEWAY.md) documents a product-local management surface that edits and reloads `[[upstream]]` config and therefore does not fit the usual `lab-apis` service shape.
