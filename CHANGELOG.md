# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased] — 0.7.0

### Added
- **Chat UI** — new `components/chat/` and `app/(admin)/chat/` for gateway-admin
- **Branding lib** — `lib/branding/` design tokens and identity helpers
- **Registry enhancements** — server detail panel expansion, filter sidebar, and richer list content
- **Admin insights** — `lib/dashboard/admin-insights.ts` with aggregated dashboard metrics
- **mcpregistry dispatch** — additional actions in `dispatch/mcpregistry/` (catalog, params, dispatch)
- **Registry type extensions** — new fields in `lib/types/registry.ts` and `lib/hooks/use-registry.ts`
- **lefthook** — pre-commit hook configuration added to `lefthook.yml`

### Changed
- **Log toolbar** — `log-toolbar.tsx` refactored; `log-filters.tsx` and `log-stream-status.tsx` removed (consolidated)
- **upstream_oauth** — `api/upstream_oauth.rs` updated with cleaner handler shape
- **MCP server** — `mcp/server.rs` updated for service consolidation
- **MCP services** — `mcp/services/tailscale.rs` and `mcp/services/tautulli.rs` deleted (fully migrated to shared dispatch layer)
- **gateway-admin** — multiple UI component updates across design system, gateway, upstream-oauth

### Version bumps
- Rust workspace: `0.6.1 → 0.7.0`
- gateway-admin: `0.1.0 → 0.2.0`

---

## [0.6.1] — 2026-04-21

| Commit | Change |
|--------|--------|
| `9d1d355` | refactor(cli): wire CLI shims to shared dispatch + add --yes/--dry-run |
| `29e6166` | fix: restore plugins/ to repo |
| `a1058de` | chore: remove stale root plugin files and gh-webhook tool |

### Highlights
- All CLI service shims now delegate to the shared `dispatch/` layer
- `--yes` / `--dry-run` flags wired for destructive actions across all services
- Plugin asset hygiene pass

---

## [0.6.0] — 2026-04-21

| Commit | Change |
|--------|--------|
| `b13fb8a` | feat(auth): browser session + upstream pool + MCP peers |
| `4ddac44` | chore(plugin): restructure plugin assets under plugins/ |

### Highlights
- Browser session cookie management for services requiring login flows
- `dispatch/upstream/pool.rs` — upstream MCP proxy pool with circuit breaker
- MCP peer registry for multi-instance upstream routing

---

## [0.5.1] — 2026-04-21

| Commit | Change |
|--------|--------|
| `beb3de0` | chore(cli): action enum validation + plugin.json simplification |
| `86ed3c5` | feat(lab-aiit.1): stdio install dispatch + security hardening for mcpregistry |

### Highlights
- CLI action enum validated at parse time (unknown actions rejected early)
- mcpregistry stdio install path + SSRF/path-traversal hardening

---

## [0.5.0] — 2026-04-21

| Commit | Change |
|--------|--------|
| `d1a3ea6` | chore: v0.5.0 — gateway-admin redesign, deploy monitor, docs |
| `740ff96` | refactor(lab-5x4t): finish aurora palette sweep |
| `513bd48` | feat(lab-5x4t.5): add --aurora-preview-* tokens |
| `6d7731d` | feat(lab-5x4t.3): migrate components/gateway to aurora tokens |
| `0f2abb7` | feat(lab-5x4t.4): migrate components/logs to aurora tokens |
| `6938158` | feat(lab-5x4t.2): migrate auth login-screen to aurora tokens |
| `3dd6734` | feat(lab-5x4t.1): add --aurora-hover-bg token |
| `0cc38fd` | refactor(lab-x2nj): move aurora tokens to components/aurora/ |
| `b37e766` | fix(lab-abch): activate shadow-aurora-* utilities |

### Highlights
- Full Aurora design token sweep across gateway-admin UI
- Aurora token module extracted to `components/aurora/tokens.ts`
- Deploy monitor scaffolding added

---

## [0.4.1] — 2026-04-21

| Commit | Change |
|--------|--------|
| `aec694f` | chore: bump version to 0.4.1 |
| `55c6c36` | feat(lab-17th.12): register CLI implementation and skill docs |
| `de0505e` | feat(lab-17th.12): register binary, systemd unit, monitor |
| `4ec80d9` | feat(lab-17th.11): axum router handlers and graceful shutdown |
| `2ececa7` | feat(lab-17th.10): flush pipeline with atomic writes and watermark |
| `58e43d7` | feat(lab-17th.9): JSONL notification line enum with atomic append |
| `bd932e4` | feat(lab-17th.8): per-PR debouncer with generation counter |
| `4744429` | feat(lab-17th.7): digest rendering with dynamic fences |
| `64fb70e` | feat(lab-17th.6): GitHub REST client with pagination + SSRF guard |
| `1d2af2a` | feat(lab-17th.5): bounded FIFO delivery-id dedup cache |
| `591b583` | feat(lab-17th.4): typed event parsing with issue_comment PR filter |
| `35372f8` | feat(lab-17th.3): constant-time HMAC-SHA256 signature verification |
| `b7f5aad` | feat(lab-17th.2): config loader with redacted Debug and empty-secret rejection |
| `6c28391` | feat(lab-17th.1): scaffold gh-webhook crate |

### Highlights
- **gh-webhook crate** — full GitHub webhook ingestion pipeline: HMAC verification, event parsing, per-PR debouncer, digest renderer, atomic JSONL append, axum HTTP server
- Bounded FIFO dedup cache for delivery-id replay protection
- GitHub REST client with SSRF guard and 429 retry

---

## [0.4.0] — 2026-04-20

| Commit | Change |
|--------|--------|
| `48ee2db` | feat(lab-eixf.8): sandbox sections + token drift docs |
| `d4f16c9` | feat(lab-eixf.7): migrate Docs page to Aurora |
| `4cf7c99` | feat(lab-eixf.6): migrate Settings page to Aurora |
| `35a4426` | feat(lab-eixf.5): migrate Activity page to Aurora |
| `ffd67c4` | feat(lab-eixf.4): migrate Overview page to Aurora |
| `d6d1c76` | feat(lab-eixf.3): Aurora primitive variants (Card/Badge/Alert) |
| `0e5c410` | simplify: abort checks, deriveGatewayName extraction |
| `ebfbab9` | fix(lab-iwtf.13,19): gateway name validation and option handling |
| `7ac4bc6` | fix(lab-iwtf.7,10,13,15): installServer return type, polling fixes |
| `9c67663` | fix(lab-iwtf.3,4,14,17,18,29): SSRF probe, restart hazard, auth edge cases |
| `d8b71eb` | fix(lab-iwtf.6,12,16): HTTP 422 for SSRF kinds, replay-window fixes |
| `10fc672` | fix(lab-iwtf.2,8): popup user-activation and external-close fixes |
| `ea21977` | fix(lab-iwtf.1,5,9,11): OAuth patch drop, proxy_prompts dedup |
| `f39f119` | feat(cli): richer palette — violet categories, teal action names |
| `806f7f9` | feat(cli): premium palette + catalog/doctor renderers |

### Highlights
- Full Aurora migration for all gateway-admin pages (Overview, Activity, Settings, Docs)
- Aurora primitive component variants (Card, Badge, Alert)
- **mcpregistry security** — SSRF probe, replay-window guard, HTTP 422 error mapping
- OAuth upstream flow fixes (popup activation, external close, proxy_prompts dedup)
- Premium CLI output palette (violet categories, teal actions, semantic colors)
