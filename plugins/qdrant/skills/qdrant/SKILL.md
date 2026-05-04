---
name: qdrant
description: Lab's Qdrant integration — Vector database for semantic search and embeddings. Use when the user wants to manage their Qdrant instance, or invokes `lab qdrant` / `mcp__lab__qdrant`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Qdrant

Vector database for semantic search and embeddings. Exposes **7 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__qdrant`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab qdrant --help
lab qdrant <action> --help
labby --json qdrant <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check Qdrant server health
- `collections.list` — List all collections
- `collections.get` — Get info about a specific collection
- `collection.create` — Create a new collection
- `collection.delete` — Delete a collection and all its data
- `point.upsert` — Insert or update points in a collection
- `point.search` — Search points by vector similarity

## Destructive actions

qdrant exposes 1 destructive action(s): `collection.delete`. These mutate state — confirm with the user before invoking. The full `Destructive` column is in `references/mcp.md`.

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service qdrant
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `qdrant`-specific.
