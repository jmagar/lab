---
name: freshrss
description: Lab's Freshrss integration — Self-hosted RSS feed aggregator. Use when the user wants to manage their Freshrss instance, or invokes `lab freshrss` / `mcp__lab__freshrss`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Freshrss

Self-hosted rss feed aggregator. Exposes **4 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__freshrss`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
lab freshrss --help
lab freshrss <action> --help
lab --json freshrss <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `subscription.list` — List all RSS feed subscriptions
- `tag.list` — List all tags/categories
- `unread.counts` — Get unread article counts per feed
- `stream.items` — Fetch articles from a feed or tag stream

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`lab extract scan` and `lab extract apply`. Verify connectivity:

```bash
lab doctor service freshrss
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `freshrss`-specific.
