---
name: immich
description: Lab's Immich integration — Self-hosted photo and video management. Use when the user wants to manage their Immich instance, or invokes `lab immich` / `mcp__lab__immich`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Immich

Self-hosted photo and video management. Exposes **6 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__immich`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab immich --help
lab immich <action> --help
labby --json immich <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check Immich server health
- `server.info` — Get Immich server info and stats
- `server.version` — Get Immich version
- `user.me` — Get current user profile
- `asset.search` — Search photos and videos
- `asset.get` — Get details for a specific asset

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service immich
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `immich`-specific.
