---
title: ACP Docs
created_at: 2026-04-23 17:03:03 EDT
updated_at: 2026-04-23 17:41:01 EDT
status: draft
owner: lab
---

# ACP

This directory is the documentation entrypoint for ACP in `lab`.

ACP is the first-class product-local service that owns conversational session
orchestration, provider runtime lifecycle, prompt execution, transcript
assembly, and event streaming.

The browser UI route remains `chat`, but the canonical backend service name is
`acp`.

## Scope

ACP covers:

- provider health and runtime lifecycle
- session creation, listing, prompting, cancellation, and resume semantics
- event persistence, replay, sequencing, and SSE delivery
- transcript-oriented session state for browser and machine-facing consumers
- provider-agnostic runtime orchestration
- raw `usage_update` preservation
- raw `ContentBlock[]` preservation and rendering
- marketplace deployment as a target for ACP/chat agents
- future CLI, MCP, and API surfaces over one shared dispatch layer

ACP does not own:

- upstream MCP configuration, discovery, auth, exposure, or routing
- browser-specific presentation concerns
- direct gateway-admin UI layout decisions

Those remain separate:

- `gateway` is the MCP control plane
- `chat` is the browser UI over ACP

## Canonical documents

- [design.md](./design.md) — formal ACP design spec and architecture record

## Current direction

The target architecture is:

`browser -> acp service -> gateway runtime -> upstream MCP`

Key locked decisions:

- canonical first-class service name: `acp`
- browser route name: `chat`
- ACP core capability logic belongs in `lab-apis`
- ACP surface adapters belong in `lab`
- ACP integrates with gateway through a narrow in-process interface
- SSE remains the default event-stream transport
- ACP runtime is provider agnostic
- minimum provider targets are Codex, Claude, Gemini, GitHub Copilot, and OpenCode
- marketplace deployments should be able to target ACP/chat agents directly
- ACP should preserve raw `usage_update` and raw `ContentBlock[]`
- ACP should invest in full `ContentBlock[]` rendering
- ACP Registry compatibility should remain a first-class direction so users can
  install additional agents/providers over time

## HTTP API

The preferred machine-facing HTTP entrypoint is the shared service dispatch
route:

```http
POST /v1/acp
Content-Type: application/json

{
  "action": "session.start",
  "params": {
    "provider": "codex",
    "cwd": "/home/example/project",
    "title": "Investigate build"
  }
}
```

The request body matches the MCP service contract: `action` is one ACP action
name from the catalog and `params` is the action-specific object. Authenticated
HTTP session actions are scoped to the caller principal by the API adapter.
Destructive actions use the shared HTTP confirmation gate and require
`params.confirm: true`.

The REST-shaped browser compatibility routes under `/v1/acp/sessions/*` remain
available for the hosted chat UI. SSE event delivery is the transport exception:
browser clients still stream events from
`GET /v1/acp/sessions/{session_id}/events?ticket=...`.

## Status

Today ACP exists as a product-local browser/API surface, but not yet as a fully
promoted first-class service across the shared `dispatch`, CLI, MCP, and
registry layers.

The design in [design.md](./design.md) is the source of truth for that
promotion.
