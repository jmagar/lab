---
name: scrutiny
description: Lab's Scrutiny integration — SMART hard drive health monitoring. Use when the user wants to manage their Scrutiny instance, or invokes `lab scrutiny` / `mcp__lab__scrutiny`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Scrutiny

Smart hard drive health monitoring. Exposes **3 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__scrutiny`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab scrutiny --help
lab scrutiny <action> --help
lab --json scrutiny <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check Scrutiny server health
- `dashboard.summary` — Get drive health dashboard summary
- `device.list` — List all monitored devices

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`lab extract scan` and `lab extract apply`. Verify connectivity:

```bash
lab doctor service scrutiny
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `scrutiny`-specific.
