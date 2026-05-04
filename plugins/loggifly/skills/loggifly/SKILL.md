---
name: loggifly
description: Lab's Loggifly integration — Docker container log alerting via keyword/regex patterns. Use when the user wants to manage their Loggifly instance, or invokes `lab loggifly` / `mcp__lab__loggifly`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Loggifly

Docker container log alerting via keyword/regex patterns. Exposes **3 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__loggifly`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab loggifly --help
lab loggifly <action> --help
labby --json loggifly <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `contract.status` — Get LoggiFly contract/API status
- `health.status` — Check LoggiFly health
- `config.summary` — Get active configuration summary

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service loggifly
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `loggifly`-specific.
