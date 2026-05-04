---
name: neo4j
description: Lab's Neo4J integration — Graph database — nodes, relationships, Cypher queries. Use when the user wants to manage their Neo4J instance, or invokes `lab neo4j` / `mcp__lab__neo4j`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Neo4J

Graph database — nodes, relationships, cypher queries. Exposes **10 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__neo4j`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab neo4j --help
lab neo4j <action> --help
labby --json neo4j <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `cypher.read` — Execute a read-only Cypher query
- `cypher.write` — Execute a write Cypher query
- `schema.labels` — List all node labels in the database
- `schema.relationships` — List all relationship types
- `schema.constraints` — List all schema constraints
- `schema.indexes` — List all indexes
- `db.list` — List available databases
- `db.info` — Get info about a specific database
- `server.status` — Get Neo4j server status
- `txn.run` — Execute a multi-statement transaction

## Destructive actions

neo4j exposes 2 destructive action(s): `cypher.write`, `txn.run`. These mutate state — confirm with the user before invoking. The full `Destructive` column is in `references/mcp.md`.

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service neo4j
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `neo4j`-specific.
