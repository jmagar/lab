# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased] — 0.10.0

| Commit | Change |
|--------|--------|
| `f8de5bde` | feat(lab-jwbg.2): migrate ACP types — Bridge* → Acp* in lab-apis |
| `bba30eb2` | feat(lab-zxx5.7): unified marketplace type filter + MCP/ACP item cards |
| `3124a871` | feat(lab-zxx5.5): add fleet WS master→device sender registry |
| `43ad105b` | fix(pr29): catalog filter chips can return to 'all' view |
| `b8ad6306` | feat(lab-zxx5.12): add Category::Marketplace, recategorize marketplace + mcpregistry |
| `35752048` | fix(pr29): address remaining review threads on AI components + docs |
| `9e0383ba` | fix(marketplace): address PR #29 review threads — installPath validation |
| `299eb724` | fix(lab-jwbg.9): eliminate try_write().expect() panic in AcpSessionRegistry |
| `526bf3e1` | feat(lab-jwbg.1): create lab-apis::acp module scaffold |
| `2013dbdd` | feat: AI component library, ACP docs, gateway/marketplace UI refinements |
| `7c4fb9f` | fix(lab-kvji.10.1): validate path components in parse_plugin_id |
| `ca66a3b` | fix(lab-kvji.10.3): validate installPath from installed_plugins.json |
| `cd8bfa9` | fix(lab-kvji.10.2): add symlink guards to all filesystem walkers |
| `a9dcd54` | Finalize gateway admin, registry, and auth follow-ups |
| `0a6c846` | feat: add registry metadata curation and admin filters |
| `479bae4` | fix: address latest PR comment |
| `5a75aba` | fix: address follow-up PR comments |
| `227b4ed` | fix: address PR review feedback |
| `fd8aafc` | docs: update fleet websocket runtime docs |
| `8ecda7b` | feat: add websocket fleet runtime |
| `facca22` | docs: add fleet ws runtime design |
| `0cad306` | Finalize remaining gateway admin and registry work |
| `47171c0` | fix: address remaining marketplace and upstream review comments |
| `4392a42` | fix: address gateway plan and docs review comments |
| `867dda3` | fix: address gateway admin design-system review comments |
| `ccafbdb` | fix: address gateway admin registry review comments |
| `91188af` | fix: address gateway admin chat and logs review comments |
| `410acdb` | Finalize remaining chat, marketplace, and deploy updates |
| `38fd124` | fix: address PR comments for gateway policy and browser session auth |
| `997110e` | fix: address PR comments for marketplace client and dialog flows |
| `6ae4bd9` | fix: address PR comments for registry and marketplace dispatch |
| `a51056f` | fix: address PR comments for gateway and registry docs |
| `e5dec3d` | Add gateway ACP, marketplace, and CLI UI updates |
| `9a0f23b` | Address PR review feedback |
| `681986c` | feat(gateway-chat-registry-log-ui): marketplace UI, gateway/chat/registry/log component polish, mcpregistry fixes — v0.7.3 |
| `802d67e` | feat(marketplace): route + sidebar nav entry — Marketplace page complete |
| `3674c5b` | feat(marketplace): all UI components — cards, panels, dialogs, modal |
| `120bf6a` | feat(marketplace): types, API client (mock data), and SWR hooks |
| `861e4e8` | feat(gateway-admin): wire listServers to GET /v0.1/servers REST endpoint |
| `de8d173` | fix(registry_v01): normalize error kinds; add owner filter; use ToolError uniformly |
| `ff6185a` | fix(mcpregistry): extract shared sync guards to dispatch layer |
| `4dfd248` | fix(mcpregistry/params): add Tailscale CGNAT range to SSRF blocklist |
| `9892d33` | fix(mcpregistry/store): ON CONFLICT DO UPDATE, jiff, WAL, UTF-8 truncation |
| `c67b839` | fix(lab): remove chrono dep, feature-gate rusqlite/r2d2 under mcpregistry |
| `281dfbd` | fix(log_fmt): replace chrono with jiff for timestamp formatting |
| `af7d12a` | fix(mcpregistry): surface upstream errors properly; add Upstream variant |
| `9ff7ded` | feat(mcpregistry): add sync observability — start/page/finish log events |
| `8e17b84` | fix(registry_v01): use axum 0.8 {param} route syntax instead of :param |
| `388c22e` | fix: squash serve/dispatch warnings (unnecessary qualifications, dead code) |
| `fca019b` | fix(gateway-admin): brand icon white bg + colored border for contrast |

### Highlights

- **Marketplace security hardening (P1)** — path traversal via plugin ID blocked at parse time; symlink following eliminated from all four filesystem walkers; `installPath` from `installed_plugins.json` validated against `plugins_root` before use
- **AI component library** — 26 new TSX components under `components/ai/` covering agents, artifacts, attachments, code blocks, reasoning, tool calls, and more
- **Fleet websocket runtime** — new `feat: add websocket fleet runtime`; ACP provider, session registry, SSE transport, and design docs
- **Registry metadata curation** — Lab-owned `_meta["tv.tootie.lab/registry"]` contract, validation, audit fields, server-side metadata filters, typed CLI metadata commands, and gateway-admin structured metadata editing
- **Gateway admin refinements** — registry detail/editor upgrades, gateway filter/table improvements, chat shell and tool-call presentation cleanup, and additional tests across chat and gateway views
- **Marketplace and upstream hardening** — marketplace client/dispatch cleanup, upstream pool adjustments, browser session auth fixes, and follow-up review-driven repairs
- **CLI and docs** — `lab mcpregistry meta get/set/delete`, metadata contract docs, and CLI coverage for the registry metadata surface
- **Marketplace UI** — full Marketplace page: types, mock API client, SWR hooks, card/panel/dialog/modal components, route + sidebar nav entry
- **Gateway admin REST wiring** — listServers now calls GET /v0.1/servers; gateway/registry/log/chat UI components updated (filters, table, detail panel, session sidebar, log console)
- **Chat UI improvements** — chat-input, chat-shell, message-bubble, message-thread, settings-panel refined; gateway tools table added
- **mcpregistry fixes** — sync guard extraction, SSRF blocklist (Tailscale CGNAT), ON CONFLICT upsert, WAL mode, jiff timestamp, upstream error surfacing, sync observability log events
- **Chrono → jiff migration** — removed chrono dep from workspace; log formatter uses jiff
- **Registry v0.1 API fixes** — axum 0.8 route syntax, owner filter, ToolError normalization, coverage doc added

### Version bumps

- Rust workspace: `0.8.0 → 0.9.0`
- gateway-admin: `0.3.0 → 0.4.0`

---

## [0.7.2] — 2026-04-22

| Commit | Change |
|--------|--------|
| `2caf21b` | feat(lab-h5pm.4): dispatch sync action with RAII AtomicBool rate-limit guard |
| `8233ac5` | feat(registry): use GitHub owner avatar as server image |
| `0d1acba` | feat(gateway-admin): aurora token sweep + eslint enforcement |
| `04a0dbd` | feat(lab-h5pm.2): implement RegistryStore query methods, upsert, and full sync |
| `96ddf66` | feat(lab-h5pm.1): create RegistryStore module skeleton in dispatch layer |

### Highlights

- **RegistryStore (lab-h5pm)** — SQLite-backed MCP server registry with skeleton, query/upsert/full-sync, and dispatch sync action protected by a RAII AtomicBool rate-limit guard
- **GitHub owner avatar** — registry list rows and detail header now pull `https://github.com/<owner>.png` from `server.repository.url`, falling back to `icons[0]` then a `Package` lucide icon
- **Aurora token sweep (product code)** — replaced shadcn-generic tokens (`text-muted-foreground`, `bg-card`, `bg-muted`, `bg-background`, `border-border`, `text-foreground`, `rounded-xl`) with Aurora equivalents across 19 files in `components/` and `app/`
- **ESLint enforcement** — new `no-restricted-syntax` rule bans the same tokens in `className` literals and template elements, scoped to `app/**` and `components/**` with `components/ui/**` exempted as the sanctioned escape hatch
- **Design-system contract** — added Authentication Surfaces section, banned-shadcn-token mapping table, eyebrow drift guidance, typography-ramp override rule, and Display Slot Assignments table
- **Brand icon polish** — gateway form brand chip now renders white-backed with colored border and SVG fill recoloring for stronger contrast
- **Test-compile repairs** — added `proxy_prompts` to `UpstreamConfig` literals across 4 files + `search` to `StoreListParams` literal; all-features tests compile clean

### Version bumps

- Rust workspace: `0.7.1 → 0.7.2`
- gateway-admin: `0.2.1 → 0.2.2`

---

## [0.7.1] — 2026-04-21

| Commit | Change |
|--------|--------|
| `52ef7d4` | refactor(ui): complete Aurora token sweep across all shadcn primitives — v0.7.1 |

### Highlights

- **Aurora token sweep** — complete theming of all `components/ui/` shadcn primitives: toggle, navigation-menu, skeleton, dialog, item, calendar, scroll-area, resizable, badge, checkbox, switch, radio-group, slider, dropdown-menu, select, alert, separator, accordion, progress, tabs, sonner, command, context-menu, menubar
- **Focus ring normalization** — all Radix primitives now use `aurora-accent-primary` rings instead of shadcn `ring-ring/50` defaults
- **Hover state normalization** — all `bg-accent`/`focus:bg-accent`/`hover:bg-accent` replaced with `aurora-hover-bg` across all menu and interactive components
- **Light mode fix** — `--aurora-hover-bg: #dcedf2` added to `.light` class (was dark-only)
- **`text-aurora-text-secondary` purge** — removed all 10 usages of the no-op token (not in `@theme inline`); replaced with `text-aurora-text-muted`
- **`aurora-scrollbar` utility** — added to `globals.css` for Firefox + WebKit scrollbar theming
- **`alert` success variant** — new `success` variant added to `alert.tsx`
- **JsonHighlight** — syntax-colored JSON renderer in `server-detail-panel.tsx`

### Version bumps

- Rust workspace: `0.7.0 → 0.7.1`
- gateway-admin: `0.2.0 → 0.2.1`

---

## [0.7.0] — 2026-04-21

| Commit | Change |
|--------|--------|
| `8cc9a59` | feat(gateway-admin): chat UI, registry enhancements, log toolbar refactor — v0.7.0 |
| `3eaa81c` | docs(observability): document ANSI sanitization, resource_uri redaction, and shell wrapper boundary |
| `762be6e` | feat(observability): add missing identifying fields to MCP/upstream warn events |
| `b09db3f` | feat(observability): normalize startup lifecycle events in lab serve |
| `0203829` | feat(formatter): extract PremiumEventFormatter into log_fmt/ with Axon-style semantic coloring |
| `234f7c4` | fix(security): sanitize log field values + redact upstream credentials |

### Highlights

- Chat UI (`components/chat/`, `app/(admin)/chat/`) and branding lib added to gateway-admin
- Registry: server detail panel expansion, filter sidebar, richer list content
- Log toolbar refactored; `log-filters.tsx` and `log-stream-status.tsx` consolidated
- Observability improvements: startup lifecycle events, MCP/upstream warn fields, ANSI sanitization
- `PremiumEventFormatter` extracted into `log_fmt/` with Axon-style semantic coloring
- Security: log field value sanitization + upstream credential redaction

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
