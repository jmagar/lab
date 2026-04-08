# Services

`lab` is built around feature-gated service integrations. Each service follows the same structural contract so the CLI, MCP server, and TUI can treat them uniformly.

## Per-Service Shape

Each service should provide:

- a `lab-apis` module
- a typed client
- request/response types
- a service-specific error type
- a `PluginMeta`
- a health-check implementation
- a CLI shim
- an MCP dispatch shim

## Feature Gates

Service integrations are feature-gated in `lab-apis` and re-exported in `lab`.

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
- `lab` defaults to the minimum useful media stack
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

The expected workflow is:

1. add the `lab-apis` module
2. define client, types, and error
3. implement health checks
4. add the feature flag to `lab-apis`
5. add the passthrough feature to `lab`
6. add CLI dispatch
7. add MCP dispatch
8. register service metadata
9. add tests

When available, OpenAPI is the preferred starting point. When it is not available, the hand-written contract should still land in the same structural shape.

The important rule is that the service client owns logic. CLI and MCP layers only adapt inputs and outputs.

## Service Inventory Direction

The project is intentionally broad but follows one rule: one binary, one consistent control plane, many integrations.

The service set is grouped conceptually, not implemented as unrelated one-offs.

## Synthetic Service

[`EXTRACT.md`](./EXTRACT.md) documents `extract`, which is not a remote API service but still follows the same structural shape for consistency.
