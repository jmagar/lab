---
name: tei
description: Lab's Tei integration — HuggingFace Text Embeddings Inference server. Use when the user wants to manage their Tei instance, or invokes `labby tei` / `mcp__lab__tei`. Calls the MCP tool first, falls back to the CLI if MCP is unavailable.
---

# Tei

Huggingface text embeddings inference server. Exposes **7 actions** via the `lab` homelab control plane.

## How to call it

**Prefer the MCP tool. Fall back to the CLI only when MCP is unavailable.**

### MCP (preferred)

One tool: `mcp__lab__tei`. Dispatch shape: `{ "action": "<name>", "params": {...} }`.

Discover actions live:
```json
{ "action": "help" }
{ "action": "schema", "params": { "action": "<name>" } }
```

Full action catalog: [`references/mcp.md`](references/mcp.md).

### CLI fallback

```bash
labby tei --help
labby tei <action> --help
labby --json tei <action> ...
```

CLI mirrors MCP actions; dots become dashes (`server.health` → `server-health`). Full CLI surface: [`references/cli.md`](references/cli.md).

## Highlights

- `server.health` — Check TEI server health
- `server.info` — Get loaded model info
- `embed.create` — Generate embeddings for text input(s)
- `embed.rerank` — Rerank texts by relevance to a query
- `embed.tokenize` — Tokenize inputs and return token counts
- `embed.similarity` — Compute pairwise similarity scores
- `embed.sparse` — Generate sparse embeddings

## Configuration

Credentials and base URLs live in `~/.lab/.env`. Onboard / re-extract with
`labby extract scan` and `labby extract apply`. Verify connectivity:

```bash
labby doctor service tei
```

## When NOT to use this skill

- The user is asking about a different lab service — load that service's skill instead.
- The user is asking about `lab` itself (CLI internals, install, gateway, doctor across all services) — that's operator-tier, not `tei`-specific.
