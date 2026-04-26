# Marketplace Catalog — Feature Design Spec

**Status:** Migrating into production `/marketplace`.
**Component:** `components/marketplace/marketplace-list-content.tsx`
**State module:** `components/marketplace/marketplace-state.ts`
**Routes:** `/marketplace` renders the migrated Marketplace catalog. `/dev/marketplace` is removed; future dev previews should use their own temporary `/dev/<feature-name>` routes and be deleted when promoted.
**Design references:** [Design System Contract](../../../docs/design/design-system-contract.md), [Component Development Process](../../../docs/design/component-development.md), and the Gateway page filter/list patterns.

## Problem And Scope

Labby admins need one catalog surface for gateway plugins, MCP servers, ACP agents, bundled agents, skills, commands, and marketplace sources. The migrated Marketplace replaces the old tab-heavy layout with a Gateway-style filter rail, summary lenses, and a density switch so large catalogs remain scannable.

## Page Structure

```text
AppHeader
  breadcrumbs: Labby / Marketplace
  actions: card view, table view, add source preview, refresh

Hero panel
  eyebrow: Plugin operations
  title: Marketplace
  description
  read-only preview notice only if temporarily rendered under /dev/*

Summary lenses
  All, Installed, Plugins, MCP servers, ACP agents, Sources

Content grid
  left: Gateway-style filter rail on desktop, collapsible filter panel on mobile
  right: results header, partial-load warning, card grid or dense table

Preview dialog
  opens for install/remove/wire/update actions
  final mutation button follows the route mode and item capability
```

## Interaction Model

The migrated Marketplace removes both old tab groups:

- `Browse / Installed / Marketplaces`
- `All / Plugins / MCP Servers / ACP Agents`

Those concepts are now handled by summary lenses and check-box filters. The lenses are high-level shortcuts, while the rail handles precise filtering.

The filter rail intentionally uses Gateway-style checkbox rows instead of pill checkboxes. This is now an approved design-system pattern for dense Marketplace/Gateway filter rails because it scans better across many filter groups and values.

Clicking a source card filters the catalog to that source instead of navigating away. Clicking plugin, MCP server, or ACP agent actions opens the review dialog. ACP copy must stay explicit: wiring an ACP implementation does not automatically make it available in `/chat` unless a separate backend flow exists.

## Data Sources

| Item type | Hook | Backend action | Dev route |
| --- | --- | --- | --- |
| Marketplace sources | `useMarketplaces()` | `sources.list` | `/dev/api/marketplace` |
| Plugin packages and bundled components | `usePlugins()` | `plugins.list` | `/dev/api/marketplace` |
| MCP servers | `useMcpServers()` | `mcp.list` | `/dev/api/marketplace` |
| ACP agents | `useAcpAgents()` | `agent.list` | `/dev/api/marketplace` |

The production API remains `/v1/marketplace`. In dev preview mode, `devPreviewActionUrl()` maps marketplace reads to `/dev/api/marketplace`.

MCP servers and ACP agents are no longer static-only UI mocks. They use live hooks and can fall back to safe local fallback data only where the hook explicitly defines fallback data.

Plugin packages are not the only catalog items derived from `plugins.list`. When a package includes `components`, Marketplace emits separate catalog rows for bundled agents, skills, commands, apps, hooks, assets, files, config, monitors, output styles, and plugin-distributed MCP servers. This prevents package contents from being counted as generic plugins.

## Filters And Sort

Filter state lives in `MarketplaceCatalogFilterState`.

| Filter | Values |
| --- | --- |
| Lens | all, installed, plugin packages, agents, skills, commands, MCP servers, ACP agents, sources |
| Search | name, subtitle, description, source, distribution, ecosystem, tags |
| Type | plugin package, agent, skill, command, MCP server, ACP agent, app, hook, asset, file, config, monitor, output style, source |
| Install state | installed, not installed, update available, built-in |
| Ecosystem | derived from loaded items |
| Source | live marketplace sources |
| Distribution | derived from loaded items |
| Sort | recently updated default, A-Z, source, installed first |

`clearFilters()` preserves the current lens and clears the detailed rail filters plus search. The default sort is recently updated so the first view emphasizes fresh plugin packages, MCP servers, and ACP agents instead of an alphabetical catalog dump.

Standalone MCP Registry rows use the synthetic source id `mcp-registry` and source name `MCP Registry`. This keeps MCP rows attached to the same source-filter model as plugin packages and prevents them from appearing as an unfilterable global block.

Sort is currently implemented as pill buttons inside the filter rail. If this becomes a dropdown or select later, it must use Aurora control surfaces, semantic borders, stable accessible naming, and the shared focus-ring tokens.

## View Modes

Desktop supports both cards and table:

- Cards are the default discovery mode and preserve the current big-card marketplace feel.
- Table mode is the condensed scan mode for dense catalogs.

Mobile uses cards only. The table toggle is hidden below `lg` because horizontal table scanning is weaker than the card stack on small screens.

## Components

| Area | Component/function |
| --- | --- |
| Page | `MarketplaceListContent` |
| Item normalization | `buildMarketplaceCatalogItems()` |
| Summary counts | `marketplaceCatalogSummary()` |
| Filtering | `filterMarketplaceCatalogItems()` |
| Sorting | `sortMarketplaceCatalogItems()` |
| Summary lens | `SummaryLens` |
| Filter checkboxes | `FilterCheckbox` |
| Cards | `CatalogCard` |
| Table | `CatalogTable` |
| Action review | shadcn `Dialog` in `MarketplaceListContent` |

## Actions

| Action | Kind | `/dev/*` behavior |
| --- | --- | --- |
| Refresh catalog | Read | Allowed |
| Search, filter, sort, switch view mode | Local state | Allowed |
| Source card click | Local filter change | Allowed |
| Add marketplace source | Mutation | Preview message only; no write sent |
| Install/update/remove plugin | Mutation | Dialog opens; final button disabled |
| Install MCP server | Mutation | Dialog opens; final button disabled |
| Wire ACP agent | Mutation | Dialog opens; final button disabled |

The production `/marketplace` route supports source-add, plugin install/remove, MCP install, ACP agent install/wiring, and bundled component cherry-pick flows through the existing Marketplace hooks and dialogs. MCP registry servers install to explicit targets: a Lab Gateway upstream, Claude/Codex MCP client config on any connected fleet device, or any combination of those targets. The modal must not present Lab service integrations such as Radarr or Sonarr as MCP install destinations.

## `/dev/*` Read-Only Enforcement

Read-only behavior is enforced in two layers.

Frontend:

- `apps/gateway-admin/lib/dev/preview-mode.ts` detects `/dev/*` for temporary previews.
- `READ_ONLY_ACTIONS` allows only whitelisted marketplace read actions.
- Mutating actions are blocked before `fetch`.
- Marketplace reads are routed from `/v1/marketplace` to `/dev/api/marketplace`.

Backend:

- `crates/lab/src/api/router.rs` mounts `POST /dev/api/marketplace` outside the authenticated `/v1/*` API.
- `DEV_MARKETPLACE_READ_ACTIONS` mirrors the frontend read whitelist.
- The handler rejects non-whitelisted actions with `403` and kind `dev_preview_read_only`.
- Allowed reads reuse the real marketplace dispatch path.

Marketplace should not depend on `/dev/marketplace`. Backend dev APIs remain available under `/dev/api/*` for temporary read-only previews.

## States

| State | Rendering |
| --- | --- |
| Populated | Card grid or table |
| Empty filters/search | Strong panel with reset guidance |
| Partial load issue | Warning panel above results |
| Refreshing | Refresh button disabled with spinner |
| Dev read-only | Hero notice plus disabled final mutation button |

There is no dedicated skeleton state yet. SWR fallback data keeps the page interactive while reads resolve.

## Responsive Rules

Desktop:

- Fixed `280px` filter rail.
- Summary lenses render in a compact wrapping strip.
- Cards use responsive columns; table mode is available.

Mobile:

- Summary lenses render as a compact grid.
- Search appears above results.
- Filter rail collapses into an inline panel opened from the search control.
- Active filter count appears on the filter button.

## Accessibility

- Summary lenses and sort pills use buttons with `aria-pressed`.
- Search inputs have names and `aria-label`s.
- View-mode buttons have `aria-label` and `aria-pressed`.
- Refresh and add-source preview controls have `aria-label`s.
- The action review uses `DialogTitle` and `DialogDescription`.
- Keyboard focus must remain visible on filters, lenses, cards, table action buttons, and dialog controls.

## Design System Compliance

Marketplace follows the design-system contract:

- `Manrope` display slots for hero and card titles.
- `Inter` UI/body text through existing application font classes.
- Aurora semantic tokens for surfaces, borders, text, accent, warning, hover, and focus states.
- Gateway-style `AURORA_MEDIUM_PANEL` and `AURORA_STRONG_PANEL` surfaces.
- `rounded-aurora-*` radii and contract spacing.
- Existing button, input, table, dialog, pill, and card primitives.
- Gateway-style checkbox-row filter rails, which are approved for dense catalog filtering in the design-system contract.

Approved deviations: none.

## Review Notes

Browser review on April 25, 2026 covered:

- desktop dark mode
- desktop light mode
- mobile dark mode
- search/filter interaction
- card/table switching
- keyboard focus from the search field
- console errors
- network routing for the preview route used during the review pass

Observed non-blocking browser behavior: desktop sidebar links can emit aborted `HEAD` prefetch requests when inspected under Playwright.

## Known Gaps

- Very large catalogs render as full lists. Add pagination or virtualization if catalog size creates measurable performance issues.
- No skeleton loading state exists yet.
- Very large catalogs render as full lists. Add pagination or virtualization if catalog size creates measurable performance issues.
- Install state for MCP servers depends on backend support; current catalog normalization treats MCP registry entries as available unless the API provides install state.
