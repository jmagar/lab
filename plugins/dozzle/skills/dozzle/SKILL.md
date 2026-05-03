---
name: dozzle
description: Lab's Dozzle integration — Real-time Docker container log viewer. Use when the user wants to manage their Dozzle instance, or invokes `lab dozzle` / `mcp__lab__dozzle`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Dozzle

Real-time docker container log viewer. Exposes **5 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__dozzle`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab dozzle --help
lab dozzle <action> --help
lab --json dozzle <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check Dozzle server health
- `server.version` — Get Dozzle version info
- `containers.list` — List all Docker containers visible to Dozzle
- `logs.fetch` — Fetch recent logs from a container
- `logs.fetch-plain` — Fetch plain-text logs from a container

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`lab extract scan` and `lab extract apply`. Verify connectivity:

```bash
lab doctor service dozzle
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `dozzle`-specific.
