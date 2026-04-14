# MCP

`lab` exposes homelab operations through a compact MCP surface designed for agents, not a giant tool registry.

The RMCP SDK integration contract that underpins this surface lives in [RMCP.md](./RMCP.md). This document owns product-facing MCP behavior; `RMCP.md` owns how `lab` uses the RMCP library to implement it.

## Transport Modes

`lab serve` supports two transports:

- `stdio`: local MCP clients such as Claude Desktop and `.mcp.json`
- `http`: remote or network-accessible MCP clients

Rules:

- `stdio` is the default
- HTTP requires a bearer token via `LAB_MCP_HTTP_TOKEN`
- transport changes must not change dispatch or catalog behavior
- HTTP transport may expose opt-in CORS origins

## Server Capabilities

`lab serve` advertises these MCP capabilities:

- tools
- resources
- prompts
- completions
- logging

Those capabilities are enabled together on the same server surface. Capability
support must reflect the running server, not a partial or hypothetical build.

## One Tool Per Service

Each service exposes exactly one MCP tool named after the service.

Examples:

```json
{ "tool": "radarr", "input": { "action": "movie.search", "params": { "query": "The Matrix" } } }
{ "tool": "plex", "input": { "action": "library.list" } }
```

This avoids exploding the tool list into hundreds of tiny tools.

Canonical service tool schema:

- `name`: service name
- `input.action`: required string
- `input.params`: optional object

## Action Model

All service tools use the same input shape:

- `action`: dotted action name such as `movie.search`
- `params`: action-specific object

Naming rule:

- lowercase
- dot-separated
- `<resource>.<verb>`

Examples:

- `movie.search`
- `queue.list`
- `system.status`

## Action Catalog

Every service declares its action catalog via `ActionSpec` and `ParamSpec`.

That catalog is the source of truth for:

- dispatch validation
- `help` action output
- MCP resources
- top-level aggregated discovery
- destructive-op policy

## Discovery

There are three discovery surfaces:

- per-service `help` action
- per-service resources such as `lab://radarr/actions`
- top-level `lab` meta-tool and `lab://catalog`

This means agents can discover the available tool shape without guessing.

Per-service resource forms:

- `lab://<service>/actions`

The top-level discovery resource is:

- `lab://catalog`

## Meta-Tool

`lab` always registers one extra tool named `lab`.

It handles:

- `help`
- `services`
- `status`
- `version`

Its job is discovery and server-level visibility, not service-specific business logic.

The top-level catalog is generated from the same action metadata that powers per-service help. It must never be maintained as a second hand-written registry.

## Result Envelope

All MCP tool responses follow a consistent envelope so callers do not need to parse arbitrary strings.

The canonical envelope and error contract lives in [SERIALIZATION.md](./SERIALIZATION.md) and [ERRORS.md](./ERRORS.md).

Success shape:

- `ok: true`
- `service`
- `action`
- `data`
- optional `meta`

Error shape:

- `ok: false`
- `service`
- `action`
- structured `error`

The envelope is intended to be the only thing an MCP client needs to parse. Multi-block or prose-heavy responses are explicitly not the default contract.

## Prompt Templates

`lab` currently exposes two prompt templates:

- `run-action`
- `service-discover`

`run-action` is the structured execution prompt. It includes:

- the selected service description when the service exists in the live registry
- the selected action description, destructive flag, return hint, and declared parameter list when the action exists
- explicit mention of the built-in `help` and `schema` actions for follow-up discovery

`service-discover` is the discovery prompt. It includes:

- the selected service description, category, and status from the live registry
- an inline action list with destructive/read-only labeling
- explicit guidance on when to call `help` or `schema`

Prompt text must be derived from the runtime registry and action metadata, not hand-maintained parallel docs.

## Completions

MCP completions are exposed for prompt and resource references, not arbitrary tool arguments.

Current completion behavior:

- prompt `service` arguments complete from the live service registry
- `run-action.action` completes from the registry-wide sorted, deduplicated action-name cache
- completion matching is simple prefix matching
- unknown prompt or resource references return empty completion sets, not errors

The cached global action-name list exists to avoid re-sorting the full action set on every completion request.

## Logging Notifications

`lab` supports MCP logging via `logging/setLevel` and server-to-client logging notifications.

Rules:

- notifications are dispatch-boundary only
- the client must opt in by calling `set_level`
- the disabled sentinel is internal-only implementation detail; clients observe standard RFC 5424 severity semantics
- success notifications emit `service`, `action`, and `elapsed_ms`
- error notifications also include a sanitized `error` string

Sanitization requirements:

- `internal_error` and `server_error` notifications must not expose raw backend details
- error strings are reduced to a single line
- home-directory paths are redacted before the notification is sent

MCP logging notifications are supplemental observability for the client session. They must never change dispatch results or leak secrets.

## Structured Error Kinds

Cross-service error vocabulary includes:

- `unknown_action`
- `unknown_subaction`
- `missing_param`
- `invalid_param`
- `unknown_instance`
- `auth_failed`
- `not_found`
- `rate_limited`
- `validation_failed`
- `network_error`
- `server_error`
- `decode_error`
- `internal_error`

Additional dispatch-level cases include:

- `elicitation_declined`
- `elicitation_unsupported`

The goal is self-correcting clients, not human-only diagnostics.

The stable semantics for these kinds are defined in [ERRORS.md](./ERRORS.md). Do not invent transport-local variants.

## Multi-Instance Services

When a service has multiple configured instances, MCP actions accept `params.instance`.

Rules:

- the dispatcher handles instance lookup
- service clients remain instance-agnostic
- unknown labels return `unknown_instance` with valid labels

## Destructive Operations

Destructive operations are marked in `ActionSpec.destructive`.

That one flag drives:

- MCP elicitation prompts
- CLI confirmation behavior

The same action metadata is used for both surfaces so the risk policy cannot drift.

Representative destructive actions include:

- container removal or stack teardown
- media deletion with file removal
- queue purge and history deletion
- network device restart or forget flows

## Elicitation Policy

MCP destructive calls require explicit confirmation unless policy is relaxed intentionally.

Modes:

- `always`
- `session`
- `never`

The default is to require confirmation, not to silently proceed.

Prompts must include:

- service
- action
- key params
- plain-language risk description

## Registry

The runtime registry only exposes enabled services. Discovery reflects the running server, not the theoretical max build.

That means:

- compiled features matter
- `--services` filtering matters
- `lab.help` only shows what is actually available

The same catalog builder must feed:

- `lab.help`
- `lab://catalog`
- CLI help/catalog rendering

## Resources

Primary resource surfaces:

- `lab://catalog`
- `lab://<service>/actions`

These are generated from the same catalog data as tool-based help.
