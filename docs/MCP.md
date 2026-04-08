# MCP

`lab` exposes homelab operations through a compact MCP surface designed for agents, not a giant tool registry.

## Transport Modes

`lab serve` supports two transports:

- `stdio`: local MCP clients such as Claude Desktop and `.mcp.json`
- `http`: remote or network-accessible MCP clients

Rules:

- `stdio` is the default
- HTTP requires a bearer token via `LAB_MCP_HTTP_TOKEN`
- transport changes should not change dispatch or catalog behavior
- HTTP transport may expose opt-in CORS origins

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
- `lab://<service>/actions/<action>`

## Meta-Tool

`lab` always registers one extra tool named `lab`.

It handles:

- `help`
- `services`
- `status`
- `version`

Its job is discovery and server-level visibility, not service-specific business logic.

The top-level catalog is generated from the same action metadata that powers per-service help. It should never be maintained as a second hand-written registry.

## Result Envelope

All MCP tool responses follow a consistent envelope so callers do not need to parse arbitrary strings.

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

Prompts should include:

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

The same catalog builder should feed:

- `lab.help`
- `lab://catalog`
- CLI help/catalog rendering

## Resources

Primary resource surfaces:

- `lab://catalog`
- `lab://<service>/actions`
- `lab://<service>/actions/<action>`

These are generated from the same catalog data as tool-based help.
