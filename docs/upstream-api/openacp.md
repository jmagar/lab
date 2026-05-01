# OpenACP Upstream REST API Contract

Retrieved: 2026-05-01

This document captures the upstream OpenACP daemon REST API contract used by
Lab's `openacp` integration. It describes the external OpenACP daemon, not
Lab's internal first-class `acp` service documented under `docs/acp/`.

## Sources

- https://raw.githubusercontent.com/Open-ACP/OpenACP/main/docs/gitbook/api-reference/rest-api.md
- https://raw.githubusercontent.com/Open-ACP/OpenACP/main/docs/gitbook/api-reference/environment-variables.md
- https://raw.githubusercontent.com/Open-ACP/OpenACP/main/docs/gitbook/api-reference/configuration-schema.md
- https://raw.githubusercontent.com/Open-ACP/OpenACP/main/docs/gitbook/self-hosting/security.md

## Service Identity

- Lab service name: `openacp`
- Env prefix: `OPENACP`
- Default upstream port: `21420`
- Conventional upstream base URL: `http://127.0.0.1:21420`
- Lab env vars: `OPENACP_URL`, `OPENACP_TOKEN`

OpenACP's own environment variables also use the `OPENACP_` prefix, but Lab
only reads `OPENACP_URL`, `OPENACP_TOKEN`, and named-instance variants such as
`OPENACP_NODE2_URL` / `OPENACP_NODE2_TOKEN`.

## Authentication

OpenACP uses `Authorization: Bearer <token>` for protected endpoints.

Token forms:

- Secret token from `<instance-root>/api-secret`, created by OpenACP with mode
  `0600`. This is a full administrative credential.
- JWT access token with scoped roles such as `admin`, `operator`, and `viewer`.

Lab does not auto-read OpenACP `api-secret` files in the first wave. Operators
must provide `OPENACP_TOKEN` explicitly.

Unauthenticated endpoints:

- `GET /api/health`
- `GET /api/version`

## Limits And Security

- Upstream documented request body limit: 1 MB.
- Default OpenACP API bind host is `127.0.0.1`.
- Upstream warns against exposing `api.host = "0.0.0.0"` without firewall
  controls.
- Swagger UI is available at `/docs` when a daemon is running.

Lab logging must not include bearer tokens, prompt bodies, raw config payloads,
workspace paths, or full session payloads.

## Endpoint Groups

### Health And System

- `GET /api/health` -> daemon health, uptime, memory, session counts, adapter
  names, and tunnel summary. No auth required.
- `GET /api/version` -> `{ "version": "..." }`. No auth required.
- `POST /api/restart` -> request daemon restart; may return `501` if restart
  is unavailable.
- `GET /api/adapters` -> registered channel adapters.

### Sessions

- `GET /api/sessions` -> session summaries.
- `GET /api/sessions/:id` -> single session details.
- `POST /api/sessions` -> create a session.
- `DELETE /api/sessions/:id` -> cancel a session.
- `POST /api/sessions/:id/prompt` -> enqueue a prompt.
- `PATCH /api/sessions/:id/bypass` -> enable or disable bypass permissions.
- `POST /api/sessions/:id/permission` -> resolve a pending permission request.
- `POST /api/sessions/:id/summary` -> ask the agent to generate a summary name.
- `POST /api/sessions/:id/archive` -> archive a session.
- `POST /api/sessions/adopt` -> adopt an external agent session.
- `GET /api/sessions/:id/config` -> session config options.
- `PUT /api/sessions/:id/config/:configId` -> update a session config option.

### Agents

- `GET /api/agents` -> configured agents and default agent.

### Configuration

- `GET /api/config` -> runtime config with upstream-redacted sensitive fields.
- `PATCH /api/config` -> update one safe config path.
- `GET /api/config/editable` -> editable config field metadata.

### Topics

- `GET /api/topics` -> topics, optionally filtered by comma-separated `status`.
- `DELETE /api/topics/:sessionId?force=true` -> delete a topic.
- `POST /api/topics/cleanup` -> delete topics matching statuses.

### Tunnel

- `GET /api/tunnel` -> primary tunnel status.
- `GET /api/tunnel/list` -> active user tunnels.
- `POST /api/tunnel` -> create a tunnel to a local port.
- `DELETE /api/tunnel/:port` -> stop one tunnel.
- `DELETE /api/tunnel` -> stop all user tunnels.

### Notifications

- `POST /api/notify` -> send a notification to registered channel adapters.

### Authentication

The upstream API also exposes token and one-time-code endpoints under
`/api/v1/auth/*`. Lab's first wave does not implement those endpoints because
they manage credentials and should be handled as a separate auth-management
slice.

### Server-Sent Events

- `GET /api/events?token=...`
- `GET /api/v1/sse/sessions/:id/stream?token=...`

Lab's first wave does not expose these SSE streams.

## Lab Product Decision

OpenACP actions are not protected by Lab's destructive-action confirmation
gate. This applies to prompt/session, bypass, permission, config, topic,
tunnel, notify, and restart actions. The OpenACP action catalog must keep every
`ActionSpec.destructive` value false unless this product decision changes.
