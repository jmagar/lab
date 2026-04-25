# Marketplace Catalog — Feature Design Spec

**Status:** Approved (retroactive — documents current implementation)
**Component:** `components/marketplace/marketplace-list-content.tsx`
**Routes:** `/marketplace` (production), `/dev/marketplace` (read-only preview)
**Design reference:** [Design System Contract](../../../docs/design/design-system-contract.md)

---

## Problem and Scope

Labby admins need a single place to discover, install, update, and remove operator tooling across three distribution formats: gateway plugins (first-party artifacts), MCP servers (npm/uvx packages for AI tool use), and ACP agents (autonomous agents for device-side operation). The catalog must support large lists with fast search and filtering, show live install state, and be reviewable in a public read-only preview without any auth.

---

## Page Structure

```
AppHeader (breadcrumb: Labby / Marketplace | actions: + Add, Refresh)
│
├── Hero panel (Display 1 heading, muted description)
│
├── Tab bar [Browse | Installed | Marketplaces] — desktop only (hidden lg:flex)
│
└── Scroll region (flex-1, overflow-y-auto)
    ├── Mobile tab chips [Browse | Installed | Sources] — mobile only (lg:hidden)
    ├── Mobile search + filter sheet (lg:hidden)
    ├── Desktop search + sort + stats strip (hidden lg:flex)
    ├── Type filter pills [All | Plugins | MCP Servers | ACP Agents]
    ├── Result count label
    └── Content grid (auto-fill, minmax(300px, 1fr))
```

---

## Tabs

| Tab | Content | Item types |
|-----|---------|-----------|
| Browse | All items, full catalog | plugins + MCP servers + ACP agents |
| Installed | Installed plugins only, grouped by marketplace source | plugins only |
| Marketplaces | Marketplace source cards | sources |

Tab switching resets the marketplace filter. Clicking a source card in Marketplaces switches to Browse with that source pre-filtered.

---

## Data Sources

| Item type | Source | Hook | Notes |
|-----------|--------|------|-------|
| Plugins | `/dev/api/marketplace` (dev) or `/v1/marketplace` (prod) via `fetchPlugins()` | `usePlugins()` | SWR, `fallbackData: []` |
| Marketplaces | Same endpoint via `fetchMarketplaces()` | `useMarketplaces()` | SWR, `fallbackData: []` |
| ACP agents | `listAcpAgents()` via `lib/marketplace/api-client` | `useAcpAgents()` | SWR, `fallbackData: MOCK_ACP_AGENTS` |
| MCP servers | `MOCK_MCP_SERVERS` constant | inline in component | Static mock, no fetch |

MCP servers are currently mocked. When a real MCP registry API is available, `MOCK_MCP_SERVERS` should be replaced with a `useMcpServers()` hook backed by `listMcpServers()`.

---

## Filters and Sort

### Type filter (browse + installed tabs)

Pills: All / Plugins / MCP Servers / ACP Agents. Counts are derived from live data lengths.

### Marketplace filter (browse tab, plugins only)

Active filter shown as a badge chip with a dismiss button on desktop. On mobile, shown as a pill in the active-filters strip. Resets on tab change.

### Search

Single text input. Matches against:
- Plugins: name, id, description, tags, marketplaceId
- MCP servers: name, description, package
- ACP agents: name, id, description

### Sort (plugins only)

| Value | Label | Behavior |
|-------|-------|---------|
| `name` | A–Z | Alphabetical by name (default) |
| `marketplace` | Marketplace | By marketplace id, then name |
| `installed` | Installed first | Installed plugins before uninstalled |
| `updated` | Recent | By `updatedAt` descending |

MCP servers and ACP agents always sort by name regardless of sort setting.

---

## View Modes

**Desktop (`lg` and up):** Inline search input + sort select + stats strip. Type filter below.

**Mobile (below `lg`):** Three stat chips for tab navigation. Collapsible filter sheet triggered by a `SlidersHorizontal` button in the search bar. Filter chip count badge on the trigger when filters are active.

---

## Cards

| Kind | Component | Key props |
|------|-----------|-----------|
| Plugin | `MarketplaceCard` | plugin, ghUser (derived from marketplace owner) |
| MCP server | `McpServerCard` | server |
| ACP agent | `AcpAgentCard` | agent |
| Marketplace source | `MktSourceCard` | marketplace, installedCount, onClick |

---

## Dialogs and Modals

| Dialog | Trigger | Mutating |
|--------|---------|---------|
| Add marketplace | `+` button in header | Yes — `sources.add` |
| Install MCP server | Install button on `McpServerCard` | Yes — `mcp.install` |
| Install ACP agent | Install button on `AcpAgentCard` | Yes — `agent.install` |
| Plugin install/uninstall | Buttons on `MarketplaceCard` | Yes — `plugin.install` / `plugin.uninstall` |

In `/dev/*` mode all mutating actions are blocked before `fetch` by `assertDevPreviewCanRunAction`. Dialogs may open but submit actions throw `DevPreviewReadOnlyError`.

---

## States

| State | Trigger | Rendering |
|-------|---------|-----------|
| Populated | Data loaded | Item grid |
| Empty search | Query with no matches | EmptyState icon 🔍 |
| Nothing installed | Installed tab, zero installed | EmptyState icon 📦 |
| Load error | `pluginsError` or `marketplacesError` | EmptyState icon ⚠️ with message |
| Refreshing | `handleRefresh` in flight | Refresh button spins, data stale |

There is no explicit skeleton loading state. SWR's `fallbackData: []` means the empty grid renders immediately, then populates when data arrives. An empty grid during initial load is acceptable and consistent with the pattern used on the Gateway list.

---

## Actions

| Action | Kind | Dev preview |
|--------|------|-------------|
| Refresh all | Read (re-fetches) | Allowed |
| Search / filter / sort | Local state | Allowed |
| Browse → Marketplaces link | Tab switch | Allowed |
| Add marketplace | Mutating | Blocked |
| Install plugin | Mutating | Blocked |
| Uninstall plugin | Mutating | Blocked |
| Install MCP server | Mutating | Blocked |
| Install ACP agent | Mutating | Blocked |

---

## Responsive Behavior

| Breakpoint | Change |
|-----------|--------|
| `< lg` | Tab chips replace tab bar; filter sheet replaces inline controls |
| `lg+` | Tab bar, inline search/sort/stats |

Grid uses `auto-fill, minmax(300px, 1fr)` — no explicit column count breakpoints.

---

## Accessibility

- Search inputs have `aria-label` and `name`
- Sort select has `aria-label`
- Filter buttons have `aria-pressed`
- Add/Refresh buttons have `aria-label`
- Mobile filter trigger has `aria-label="Open filters"`
- Tab chips and tab bar buttons are `<button type="button">`

---

## Design System Compliance

- Typography: `font-display font-extrabold` for hero and metric displays; `font-sans` for UI
- Colors: Aurora semantic tokens throughout; no raw hex except for `color-mix` composites on accent backgrounds
- Elevation: hero panel uses `AURORA_MEDIUM_PANEL`; filter sheet uses `AURORA_MEDIUM_PANEL`
- Radius: `rounded-aurora-1`, `rounded-aurora-2`
- Scrollbar: `aurora-scrollbar` class on the main scroll region
- Pill active state: `pillTone(true/false)` from aurora tokens

---

## Dev Preview (`/dev/marketplace`)

Route: `app/dev/marketplace/page.tsx` — renders `<MarketplaceListContent />` directly.

Layout: `app/dev/layout.tsx` wraps with `AppSidebar` + dev preview banner. `AuthBootstrap` is not present in this layout, so the session store remains at `loading` state. `AppSidebar` handles this correctly: the user card in the footer only renders for `authenticated` status, so unauthenticated visitors see clean navigation and theme toggle with no network errors.

Data: routes through `devPreviewActionUrl()` to `/dev/api/marketplace` (Rust backend, no auth required). Read-only action whitelist enforced at both frontend and backend layers.

---

## Known Gaps and Future Work

- MCP servers are mocked. Replace `MOCK_MCP_SERVERS` with a live `useMcpServers()` hook when the MCP registry API is available.
- The Installed tab only shows plugins. When MCP server and ACP agent install tracking is added, extend the installed view to include them.
- No skeleton loading state. Add skeletons when backend latency is a concern.
- Plugin detail view (navigate to individual plugin) is handled by `plugin-detail-content.tsx` but not yet linked from the catalog cards — the `MarketplaceCard` component does not have an href to the detail page.

---

## Deviations from Design System Contract

None approved. The implementation follows the design system contract throughout.
