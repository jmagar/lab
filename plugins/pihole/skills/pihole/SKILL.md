---
name: pihole
description: Lab's Pihole integration — Network-wide DNS ad blocking. Use when the user wants to manage their Pihole instance, or invokes `lab pihole` / `mcp__lab__pihole`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Pihole

Network-wide dns ad blocking. Exposes **10 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__pihole`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab pihole --help
lab pihole <action> --help
labby --json pihole <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.summary` — Get Pi-hole summary statistics
- `server.settings` — Get Pi-hole server settings
- `blocking.status` — Get current blocking enabled/disabled state
- `blocking.set` — Enable or disable blocking (with optional timer)
- `querylog.search` — Search the DNS query log
- `adlist.list` — List all adlists
- `adlist.add` — Add a new adlist by URL
- `adlist.remove` — Remove an adlist by ID
- `domain.list` — List allow/deny list domains
- `domain.add` — Add a domain to allow or deny list

## Destructive actions

pihole exposes 4 destructive action(s): `blocking.set`, `adlist.add`, `adlist.remove`, `domain.add`. These mutate state — confirm with the user before invoking. The full `Destructive` column is in `references/mcp.md`.

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service pihole
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `pihole`-specific.
