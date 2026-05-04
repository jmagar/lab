---
name: glances
description: Lab's Glances integration — Cross-platform system monitoring dashboard. Use when the user wants to manage their Glances instance, or invokes `lab glances` / `mcp__lab__glances`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Glances

Cross-platform system monitoring dashboard. Exposes **10 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__glances`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab glances --help
lab glances <action> --help
labby --json glances <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check Glances API health
- `system.info` — Get host system info
- `quicklook.summary` — Get CPU/mem/load quicklook summary
- `cpu.summary` — Get CPU usage summary
- `memory.summary` — Get memory usage summary
- `load.summary` — Get system load averages
- `network.summary` — Get network interface stats
- `diskio.summary` — Get disk I/O stats
- `fs.summary` — Get filesystem usage summary
- `process.top` — Get top processes by resource usage

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service glances
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `glances`-specific.
